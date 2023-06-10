use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let cwd = env::current_dir().unwrap();

    fs::create_dir_all(&out_dir).expect("Failed to create man directory");

    let src_path = Path::new(&cwd).join("man").join("man1").join("quartz.1");
    let dest_path = Path::new(&out_dir).join("quartz.1");

    fs::copy(&src_path, &dest_path).expect("Failed to copy manpage file");
}
