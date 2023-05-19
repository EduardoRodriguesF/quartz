use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use crate::internals::layout;

pub struct Endpoint {
    pub name: String,
    pub req: hyper::Request<()>,
}

impl Endpoint {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            req: hyper::Request::new(()),
        }
    }

    pub fn dir(&self) -> PathBuf {
        layout::which_dir().join(&self.name)
    }

    pub fn parse_from(name: &str) -> Self {
        let mut builder = hyper::Request::builder();
        let mut endpoint = Self::new(name);

        // Resolve headers
        if let Ok(lines) = read_lines(endpoint.dir().join("headers")) {
            for line in lines {
                if let Ok(header) = line {
                    let splitted_header = header.splitn(2, ": ").collect::<Vec<&str>>();

                    let key = splitted_header.get(0);
                    let value = splitted_header.get(1).unwrap_or_else(|| &"");

                    println!("setting key {:?} with value {:?}", key, value);

                    if let Some(key) = key {
                        builder = builder.header(*key, *value);
                    }
                }
            }
        }

        endpoint.req = builder.body(()).unwrap();

        endpoint
    }

    /// Records files to build this endpoint with `parse` methods.
    pub fn write(&self) {
        let dir = self.dir();

        // Tries to create directory for endpoint
        if !std::fs::create_dir(&dir).is_ok() {
            eprintln!("Failed to create endpoint");
            return;
        }

        let headers = self.req.headers();

        // Write /endpoint/headers
        if !headers.is_empty() {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(dir.join("headers"))
            {
                for (key, value) in headers {
                    let _ = file.write(format!("{}: {}", key.as_str(), value.to_str().unwrap()).as_bytes());
                }
            }
        }

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(dir.join("uri"))
        {
            let method = self.req.method().to_string();

            let _ = file.write(format!("{} {}", method, self.req.uri().to_string()).as_bytes());
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
