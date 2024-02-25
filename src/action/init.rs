use crate::QuartzResult;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use colored::Colorize;

#[derive(clap::Args, Debug)]
pub struct Args {
    directory: Option<PathBuf>,
}

pub fn cmd(args: Args) -> QuartzResult {
    let directory = args.directory.unwrap_or(Path::new(".").to_path_buf());
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
        "env",
        "env/default",
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
            let _ = gitignore.write("\n# Quartz\n.quartz/user\n.quartz/env/**/cookies".as_bytes());
        }
    }

    println!(
        "Initialized quartz project in {}",
        std::fs::canonicalize(quartz_dir.clone())
            .unwrap_or(quartz_dir)
            .to_str()
            .unwrap()
    );

    Ok(())
}
