use std::env;
use std::path::Path;

fn main() {
    let mut doc = pandoc::new();

    let cwd = env::current_dir().unwrap();

    let src_path = Path::new(&cwd).join("doc").join("quartz.1.md");
    let dest_path = Path::new(&cwd).join("doc").join("quartz.1");

    doc.add_input(&src_path);
    doc.set_output(pandoc::OutputKind::File(dest_path));
    doc.add_option(pandoc::PandocOption::Standalone);

    doc.execute().expect("Failed to compile manpage");
}
