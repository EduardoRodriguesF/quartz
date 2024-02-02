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
    pub multiline: bool,
}

impl Curl {
    pub async fn print(
        &self,
        handle: &EndpointHandle,
        endpoint: &Endpoint,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let mut has_printed_data = false;
        if let Some(chunk) = endpoint.body(&handle).data().await {
            if let Ok(mut chunk) = chunk {
                if !has_printed_data {
                    print!("{}{} '", separator, self.option_string(CurlOption::Data));
                    has_printed_data = true;
                }

                if chunk.ends_with("\n".as_bytes()) {
                    chunk.truncate(chunk.len() - 1);
                }

                stdout().write_all(&chunk)?;
            }
        }

        if has_printed_data {
            println!("'");
        } else {
            println!();
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
    pub async fn print(
        handle: &EndpointHandle,
        endpoint: &Endpoint,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = endpoint.full_url()?;
        let path = url.path_and_query().unwrap();

        println!("{} {} HTTP/1.1", endpoint.method, path.as_str());
        println!("Host: {}", url.host().unwrap());
        println!("{}", endpoint.headers);

        if let Some(chunk) = endpoint.body(&handle).data().await {
            if let Ok(chunk) = chunk {
                stdout().write_all(&chunk)?;
            }
        }

        Ok(())
    }
}
