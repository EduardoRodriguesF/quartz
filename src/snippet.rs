use std::ops::Deref;

use crate::{form::Form, Endpoint, QuartzResult};
use hyper::{Body, Request, Response};

enum CurlOption {
    Location,
    Request,
    Header,
    Data,
    Form,
}

#[derive(clap::Args, Debug)]
pub struct Curl {
    /// Use long form cURL options (--header instead of -H)
    #[arg(long)]
    long: bool,

    /// Split output across multiple lines
    #[arg(long)]
    multiline: bool,
}

impl Curl {
    pub fn print(&self, endpoint: &mut Endpoint) -> QuartzResult {
        let separator = if self.multiline { " \\\n\t" } else { " " };
        let mut is_form_data = false;

        print!(
            "curl {} '{}'",
            self.arg_str(CurlOption::Location),
            endpoint.full_url().unwrap()
        );
        print!(" {} {}", self.arg_str(CurlOption::Request), endpoint.method);

        for (key, value) in endpoint.headers.iter() {
            if key.to_lowercase() == "content-type" && value.starts_with("multipart/form-data") {
                // Do not show this header
                is_form_data = true;
                continue;
            }

            print!(
                "{}{} '{}: {}'",
                separator,
                self.arg_str(CurlOption::Header),
                key,
                value
            );
        }

        if let Some(body) = endpoint.body() {
            if is_form_data {
                let form = Form::from(body.as_str());

                for field in form.fields {
                    print!(
                        "{}{} {}={}",
                        separator,
                        self.arg_str(CurlOption::Form),
                        field.name,
                        field.value
                    );
                }
            } else {
                let mut body = body.to_owned();
                print!("{}{} '", separator, self.arg_str(CurlOption::Data));

                if body.ends_with('\n') {
                    body.truncate(body.len() - 1);
                }

            if body.ends_with('\n') {
                body.truncate(body.len() - 1);
            }

            print!("{body}");
            println!("'");
        } else {
            println!();
        }

        println!();

        Ok(())
    }

    fn arg_str(&self, option: CurlOption) -> &str {
        match option {
            CurlOption::Location => {
                if self.long {
                    "--location"
                } else {
                    "-L"
                }
            }
            CurlOption::Request => {
                if self.long {
                    "--request"
                } else {
                    "-X"
                }
            }
            CurlOption::Header => {
                if self.long {
                    "--header"
                } else {
                    "-H"
                }
            }
            CurlOption::Data => {
                if self.long {
                    "--data"
                } else {
                    "-d"
                }
            }
            CurlOption::Form => {
                if self.long {
                    "--form"
                } else {
                    "-F"
                }
            }
        }
    }
}

pub struct Http(String);

impl Deref for Http {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Response<Body>> for Http {
    fn from(value: &Response<Body>) -> Self {
        let mut output = String::new();

        output.push_str(&format!("< {:?}", value.version()));
        output.push_str(&format!(" {:?}", value.status()));
        output.push('\n');

        for (k, v) in value.headers().iter() {
            output.push_str(&format!(
                "< {}: {}\n",
                k.as_str(),
                v.to_str().unwrap_or_default()
            ))
        }

        output.push('<');

        Self(output)
    }
}

impl From<&Request<Body>> for Http {
    fn from(value: &Request<Body>) -> Self {
        let mut output = String::new();

        output.push_str(&format!(
            "> {} {} {:?}\n",
            value.method(),
            value.uri().path_and_query().unwrap().as_str(),
            value.version()
        ));
        output.push_str(&format!("> Host: {}\n", value.uri().host().unwrap()));

        for (k, v) in value.headers().iter() {
            output.push_str(&format!(
                "> {}: {}\n",
                k.as_str(),
                v.to_str().unwrap_or_default()
            ))
        }

        output.push('>');

        Self(output)
    }
}

impl Http {
    pub fn print(endpoint: &mut Endpoint) -> QuartzResult {
        let url = endpoint.full_url()?;
        let path = url.path_and_query().unwrap();

        println!("{} {} HTTP/1.1", endpoint.method, path.as_str());
        println!("Host: {}", url.host().unwrap());
        print!("{}", endpoint.headers);

        if let Some(body) = endpoint.body() {
            if !body.starts_with("\r\n") {
                println!();
            }

            print!("{body}");
        }

        Ok(())
    }
}
