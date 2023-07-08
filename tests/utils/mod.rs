use std::default::Default;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub struct Quartz {
    bin: PathBuf,
    pub tmpdir: PathBuf,
}

impl Default for Quartz {
    fn default() -> Self {
        let tmpdir = std::env::temp_dir();
        let bin = std::fs::canonicalize(Path::new("target").join("debug").join("quartz"))
            .expect("Failed to get binary");

        Quartz { tmpdir, bin }
    }
}

impl Drop for Quartz {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir()).unwrap();
    }
}

impl Quartz {
    pub fn cmd<S>(&self, args: &[S]) -> Result<std::process::Child, std::io::Error>
    where
        S: AsRef<OsStr>,
    {
        Command::new(self.bin.as_path())
            .current_dir(self.tmpdir.as_path())
            .args(args)
            .spawn()
    }

    pub fn dir(&self) -> PathBuf {
        self.tmpdir.join(".quartz")
    }
}
