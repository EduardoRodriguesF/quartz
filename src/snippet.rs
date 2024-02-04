use crate::{Endpoint, EndpointHandle, QuartzResult};

enum CurlOption {
    Location,
    Request,
    Header,
    Data,
}

#[derive(Default)]
pub struct Curl {
    pub long: bool,
    pub multiline: bool,
}

impl Curl {
    pub fn print(&self, handle: &EndpointHandle, endpoint: &Endpoint) -> QuartzResult {
        let separator = if self.multiline { " \\\n\t" } else { " " };

        print!(
            "curl {} '{}'",
            self.option_string(CurlOption::Location),
            endpoint.full_url().unwrap()
        );
        print!(
            " {} {}",
            self.option_string(CurlOption::Request),
            endpoint.method
        );

        for (key, value) in endpoint.headers.iter() {
            print!(
                "{}{} '{}: {}'",
                separator,
                self.option_string(CurlOption::Header),
                key,
                value
            );
        }

        let mut body = endpoint.body(&handle);
        if !body.is_empty() {
            print!("{}{} '", separator, self.option_string(CurlOption::Data));

            if body.ends_with("\n") {
                body.truncate(body.len() - 1);
            }

            print!("{body}");
            println!("'");
        }

        Ok(())
    }

    fn option_string(&self, option: CurlOption) -> String {
        let result = match option {
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
        };

        result.to_string()
    }
}

pub struct Http;

impl Http {
    pub fn print(handle: &EndpointHandle, endpoint: &Endpoint) -> QuartzResult {
        let url = endpoint.full_url()?;
        let path = url.path_and_query().unwrap();

        println!("{} {} HTTP/1.1", endpoint.method, path.as_str());
        println!("Host: {}", url.host().unwrap());
        print!("{}", endpoint.headers);

        if endpoint.has_body(&handle) {
            println!()
        }

        if endpoint.has_body(&handle) {
            let body = endpoint.body(&handle);
            print!("{body}");
        }

        Ok(())
    }
}
