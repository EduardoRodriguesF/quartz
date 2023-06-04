use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub fn state_file_path() -> PathBuf {
    Path::new(".quartz")
        .join("user")
        .join("state")
        .join("endpoint")
}

pub fn read_state() -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(state_file_path())
}

pub fn update_state(endpoint: &str) -> Result<(), std::io::Error> {
    let state_file = std::fs::OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(state_file_path());

    state_file.unwrap().write_all(endpoint.as_bytes())
}
