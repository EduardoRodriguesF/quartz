use crate::{cookie::Cookie, Ctx};

#[derive(clap::Args, Debug)]
pub struct PrintArgs {
    key: Option<String>,

    /// Filter cookies that match this domain
    #[arg(long, short = 'd')]
    domain: Option<String>,
}

pub fn print(ctx: &Ctx, args: PrintArgs) {
    let jar = ctx.require_env().cookie_jar(ctx);

    let iter = jar.iter().filter(|c| {
        if let Some(domain) = &args.domain {
            c.domain().matches(domain.as_str())
        } else {
            true
        }
    });

    if let Some(key) = args.key {
        let cookies = iter.filter(|c| c.name() == key).collect::<Vec<&Cookie>>();

        match cookies.len() {
            0 => panic!("{key}: No such cookie"),
            1 => println!("{}", cookies[0].value()),
            _ => {
                for c in cookies {
                    println!("{}: {}", **c.domain(), c.value());
                }
            }
        }
    } else {
        for cookie in iter {
            println!("{}={}", cookie.name(), cookie.value());
        }
    }
}
