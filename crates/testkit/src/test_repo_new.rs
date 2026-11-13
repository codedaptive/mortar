//! TestRepo::new performs real setup I/O; tests/support.rs covers init and drop (S-5).

use crate::TestRepo;
use std::io;
use std::process::Command;

impl TestRepo {
    /// Makes a SHA-1 repository on branch main, independent of a caller's git-hook environment.
    pub fn new() -> io::Result<Self> {
        let directory = tempfile::Builder::new().prefix("mortar-test-").tempdir()?;
        let path = directory.path().to_path_buf();
        let mut command = Command::new("git");
        command.arg("-C").arg(&path).args(["init", "-b", "main", "--object-format=sha1"]);
        // Setup must target this empty fixture even when the test runs inside a hook.
        for name in [
            "GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX",
            "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        ] {
            command.env_remove(name);
        }
        let output = command
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("LC_ALL", "C")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "git init in {}: {}: {}", path.display(), output.status,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(Self { path, directory })
    }
}
