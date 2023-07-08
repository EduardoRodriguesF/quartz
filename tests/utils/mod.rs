use std::default::Default;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::time::SystemTime;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct Quartz {
    bin: PathBuf,
    tmpdir: PathBuf,
}

pub struct QuartzOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
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
    pub fn cmd<S>(&self, args: &[S]) -> Result<QuartzOutput, std::io::Error>
    where
        S: AsRef<OsStr>,
    {
        let output = Command::new(self.bin.as_path())
            .current_dir(self.tmpdir.as_path())
            .args(args)
            .output()?;

        Ok(QuartzOutput {
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
            status: output.status,
        })
    }

    pub fn dir(&self) -> PathBuf {
        self.tmpdir.join(".quartz")
    }
}
