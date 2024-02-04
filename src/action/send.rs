use crate::{
    endpoint::{EndpointInput, Headers},
    history::{self, HistoryEntry},
    Ctx, PairMap, QuartzResult,
};
use hyper::{
    body::{Bytes, HttpBody},
    Body, Client, Uri,
};
use std::str::FromStr;
use tokio::{
    io::{stdout, AsyncWriteExt as _},
    time::Instant,
};

#[derive(Default)]
pub struct Args {
    pub headers: Vec<String>,
    pub query: Vec<String>,
    pub variables: Vec<String>,
    pub request: Option<String>,
    pub data: Option<String>,
    pub no_follow: bool,
}

pub async fn cmd(ctx: &Ctx, args: Args) -> QuartzResult {
    let (handle, mut endpoint) = ctx.require_endpoint();
    let mut context = ctx.require_context();
    for var in args.variables {
        context.variables.set(&var);
    }

    if !endpoint.headers.contains_key("user-agent") {
        endpoint
            .headers
            .insert("user-agent".to_string(), Ctx::user_agent());
    }

    endpoint.update(&mut EndpointInput {
        headers: args.headers,
        query: args.query,
        method: args.request,
        ..Default::default()
    });

    endpoint.apply_context(&context);

    let raw_body = args.data.unwrap_or(endpoint.body(&handle));
    let mut start: Instant;
    let mut res: hyper::Response<Body>;
    let mut duration: u64;
    loop {
        let req = endpoint
            // TODO: Find a way around this clone
            .clone()
            .into_request(&handle)
            .unwrap_or_else(|_| panic!("malformed request"));

        let client = {
            let https = hyper_tls::HttpsConnector::new();
            Client::builder().build(https)
        };

        start = Instant::now();
        res = client.request(req).await?;
        duration = start.elapsed().as_millis() as u64;

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

    let status = res.status().as_u16();

    let mut bytes = Bytes::new();
    let mut size = 0;

    while let Some(chunk) = res.data().await {
        if let Ok(chunk) = chunk {
            size += chunk.len();
            bytes = [bytes, chunk].concat().into();
        }
    }

    let entry: HistoryEntry = {
        let mut headers = Headers::default();
        for (key, value) in res.headers() {
            headers.insert(key.to_string(), String::from(value.to_str().unwrap_or("")));
        }

        let req_body_bytes = hyper::body::to_bytes(raw_body).await?;

        let request = history::Request {
            endpoint,
            context,
            duration,
            body: String::from_utf8_lossy(&req_body_bytes).to_string(),
        };
        let response = history::Response {
            status,
            size,
            body: String::from_utf8_lossy(&bytes).to_string(),
            headers,
        };

        HistoryEntry::new(handle.handle(), request, response)
    };

    let _ = stdout().write_all(&bytes).await;
    let _ = entry.write();

    Ok(())
}
