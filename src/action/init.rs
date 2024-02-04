use crate::{Context, QuartzResult};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use colored::Colorize;

pub fn cmd(dir: Option<PathBuf>) -> QuartzResult {
    let directory = dir.unwrap_or(Path::new(".").to_path_buf());
    let quartz_dir = directory.join(".quartz");

    if quartz_dir.exists() {
        panic!(
            "quartz already initialized at {}",
            directory.to_string_lossy()
        );
    }

    if std::fs::create_dir(&quartz_dir).is_err() {
        panic!("failed to initialize quartz project");
    };

    let ensure_dirs = vec![
        "endpoints",
        "user",
        "user/history",
        "user/state",
        "contexts",
    ];

    for dir in ensure_dirs {
        if std::fs::create_dir(quartz_dir.join(PathBuf::from_str(dir)?)).is_err() {
            panic!("failed to create {} directory", dir);
        }
    }

    if directory.join(".git").exists() {
        println!("Git detected");
        println!("Adding user files to {}", ".gitignore".green());

        if let Ok(mut gitignore) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(directory.join(".gitignore"))
        {
            let _ = gitignore.write("\n# Quartz\n.quartz/user".as_bytes());
        }
    }

    if Context::default().write().is_err() {
        panic!("failed to create default context");
    }

    Ok(())
}
