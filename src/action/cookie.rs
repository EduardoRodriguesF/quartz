use crate::{cookie::Cookie, Ctx};

#[derive(Default)]
pub struct CookieArgs {
    pub domain: Option<String>,
}

pub fn get(ctx: &Ctx, key: String, args: CookieArgs) {
    let jar = ctx.require_env().cookie_jar(ctx);

    let cookies = jar
        .iter()
        .filter(|c| {
            if let Some(domain) = &args.domain {
                c.domain().matches(domain.as_str())
            } else {
                true
            }
        })
        .filter(|c| c.name() == key)
        .collect::<Vec<&Cookie>>();

    match cookies.len() {
        0 => panic!("{key}: No such cookie"),
        1 => println!("{}", cookies[0].value()),
        _ => {
            for c in cookies {
                println!("{}: {}", **c.domain(), c.value());
            }
        }
    }
}

pub fn ls(ctx: &Ctx, args: CookieArgs) {
    let jar = ctx.require_env().cookie_jar(ctx);
    let cookies = jar
        .iter()
        .filter(|c| {
            if let Some(domain) = &args.domain {
                c.domain().matches(domain.as_str())
            } else {
                true
            }
        })
        .collect::<Vec<&Cookie>>();

    for cookie in cookies {
        println!("{}={}", cookie.name(), cookie.value());
    }
}
