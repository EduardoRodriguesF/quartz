use std::io::{BufRead, Write};

pub fn create(name: &str) {
    std::fs::create_dir_all(format!("./.api-prototype/layouts/{}", name))
        .expect(&format!("Could not create layout: {}", name));
}

pub fn list() -> Vec<String> {
    if let Ok(files) = std::fs::read_dir("./.api-prototype/layouts") {
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
    let file = std::fs::File::open("./.api-prototype/state").unwrap();
    let mut layout = String::new();

    let mut reader = std::io::BufReader::new(file);
    let _ = reader.read_line(&mut layout);

    layout
}

pub fn switch(layout: &String) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./.api-prototype/state")
        .unwrap();

    if let Ok(()) = file.write_all(layout.as_bytes()) {
        println!("Switched to {} layout.", layout);
    } else {
        eprintln!("Failed to switch to {} layout.", layout);
    };
}
