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

pub fn switch(layout: &String) {
    println!("switched to {}", layout);
}
