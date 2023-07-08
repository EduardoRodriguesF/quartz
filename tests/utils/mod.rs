use std::default::Default;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct Quartz {
    bin: PathBuf,
    tmpdir: PathBuf,
}

impl Default for Quartz {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let tmpdir = std::env::temp_dir()
            .join("quartz_cli_tests")
            .join(now.as_millis().to_string());
        let bin = std::fs::canonicalize(Path::new("target").join("debug").join("quartz"))
            .expect("Failed to get binary");

        std::fs::create_dir_all(&tmpdir).unwrap();

        Quartz { tmpdir, bin }
    }
}

impl Drop for Quartz {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir()).unwrap();
    }
}

impl Quartz {
    pub fn cmd<S>(&self, args: &[S]) -> Result<std::process::ExitStatus, std::io::Error>
    where
        S: AsRef<OsStr>,
    {
        let mut command = Command::new(self.bin.as_path())
            .current_dir(self.tmpdir.as_path())
            .args(args)
            .spawn()?;

        command.wait()
    }

    pub fn dir(&self) -> PathBuf {
        self.tmpdir.join(".quartz")
    }
}
