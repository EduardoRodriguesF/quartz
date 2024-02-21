use crate::{
    cookie::CookieJar,
    endpoint::EndpointInput,
    history::{self, History},
    Ctx, PairMap, QuartzResult,
};
use chrono::Utc;
use hyper::{
    body::{Bytes, HttpBody},
    Body, Client, Uri,
};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::io::{stdout, AsyncWriteExt as _};

#[derive(Default)]
pub struct Args {
    pub headers: Vec<String>,
    pub query: Vec<String>,
    pub variables: Vec<String>,
    pub request: Option<String>,
    pub data: Option<String>,
    pub no_follow: bool,
    pub cookies: Vec<String>,
    pub cookie_jar: Option<PathBuf>,
}

pub async fn cmd(ctx: &Ctx, args: Args) -> QuartzResult {
    let (handle, mut endpoint) = ctx.require_endpoint();
    let mut env = ctx.require_env();
    for var in args.variables {
        env.variables.set(&var);
    }

    if !endpoint.headers.contains_key("user-agent") {
        endpoint
            .headers
            .insert("user-agent".to_string(), Ctx::user_agent());
    }

    let mut cookie_jar = env.cookie_jar(ctx);

    let extras = args
        .cookies
        .iter()
        .map(|c| {
            if c.contains('=') {
                return vec![c.to_owned()];
            }

            let path = Path::new(c);
            if !path.exists() {
                panic!("no such file: {c}");
            }

            CookieJar::read(&path)
                .unwrap()
                .iter()
                .map(|c| format!("{}={}", c.name(), c.value()))
                .collect()
        })
        .flatten();

    let cookie_value = cookie_jar
        .iter()
        .map(|c| format!("{}={}", c.name(), c.value()))
        .chain(extras)
        .collect::<Vec<String>>()
        .join("; ");

    endpoint
        .headers
        .insert(String::from("Cookie"), cookie_value);

    endpoint.update(&mut EndpointInput {
        headers: args.headers,
        query: args.query,
        method: args.request,
        ..Default::default()
    });

    endpoint.apply_env(&env);

    let raw_body = args.data.unwrap_or(endpoint.body());
    let mut res: hyper::Response<Body>;
    let mut entry = history::Entry::builder();
    entry
        .handle(handle.handle())
        .timestemp(Utc::now().timestamp_micros());

    loop {
        let req = endpoint
            // TODO: Find a way around this clone
            .clone()
            .into_request(raw_body.clone())
            .unwrap_or_else(|_| panic!("malformed request"));

        entry.message(&req);
        if !raw_body.is_empty() {
            entry.message_raw(raw_body.clone());
        }

        let client = {
            let https = hyper_tls::HttpsConnector::new();
            Client::builder().build(https)
        };

        res = client.request(req).await?;

        entry.message(&res);

        if let Some(cookie_header) = res.headers().get("Set-Cookie") {
            let url = endpoint.full_url()?;

            cookie_jar.set(url.host().unwrap(), cookie_header.to_str()?);
        }

        if args.no_follow || !res.status().is_redirection() {
            break;
        }

        if let Some(location) = res.headers().get("Location") {
            let location = location.to_str()?;

            if location.starts_with('/') {
                let url = endpoint.full_url()?;
                // This is awful
                endpoint.url = Uri::builder()
                    .authority(url.authority().unwrap().as_str())
                    .scheme(url.scheme().unwrap().as_str())
                    .path_and_query(location)
                    .build()?
                    .to_string();
            } else {
                if Uri::from_str(location).is_ok() {
                    endpoint.url = location.to_string();
                }
            }
        };
    }

    match args.cookie_jar {
        Some(path) => cookie_jar.write_at(&path)?,
        None => cookie_jar.write()?,
    };

    let mut bytes = Bytes::new();

    while let Some(chunk) = res.data().await {
        if let Ok(chunk) = chunk {
            bytes = [bytes, chunk].concat().into();
        }
    }

    entry.message_raw(String::from_utf8(bytes.to_vec())?);

    let _ = stdout().write_all(&bytes).await;
    History::write(ctx, entry.build()?)?;

    Ok(())
}
