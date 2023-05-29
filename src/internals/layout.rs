use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub fn create(name: &str) {
    std::fs::create_dir_all(format!("./.quartz/layouts/{}", name))
        .unwrap_or_else(|_| panic!("Could not create layout: {}", name));
}

pub fn list() -> Vec<String> {
    if let Ok(files) = std::fs::read_dir("./.quartz/layouts") {
        return files
            .map(|f| f.unwrap().file_name().to_str().unwrap().to_string())
            .collect();
    };

    Vec::<String>::new()
}

pub fn does_exist(layout: &String) -> bool {
    list().contains(layout)
}

/// Returns currently active layout.
pub fn which() -> String {
    let file = std::fs::File::open("./.quartz/state").unwrap();
    let mut layout = String::new();

    let mut reader = std::io::BufReader::new(file);
    let _ = reader.read_line(&mut layout);

    layout
}

pub fn which_dir() -> PathBuf {
    Path::new(".quartz").join("layouts").join(which())
}

pub fn switch(layout: &String) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./.quartz/state")
        .unwrap();

    if let Ok(()) = file.write_all(layout.as_bytes()) {
        println!("Switched to {} layout.", layout);
    } else {
        eprintln!("Failed to switch to {} layout.", layout);
    };
}
