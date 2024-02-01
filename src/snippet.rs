use crate::{Endpoint, EndpointHandle};
use hyper::body::HttpBody;
use std::io::{stdout, Write};

enum CurlOption {
    Location,
    Request,
    Header,
    Data,
}

#[derive(Default)]
pub struct Curl {
    pub long: bool,
}

impl Curl {
    pub async fn print(&self, handle: &EndpointHandle, endpoint: &Endpoint) {
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
                " {} '{}: {}'",
                self.option_string(CurlOption::Header),
                key,
                value
            );
        }

        let mut has_printed_data = false;
        if let Some(chunk) = endpoint.body(&handle).data().await {
            if !has_printed_data {
                print!(" {} '", self.option_string(CurlOption::Data));
                has_printed_data = true;
            }

            let mut chunk = chunk.unwrap();
            if chunk.ends_with("\n".as_bytes()) {
                chunk.truncate(chunk.len() - 1);
            }

            stdout().write_all(&chunk).unwrap();
        }

        if has_printed_data {
            println!("'");
        }
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
