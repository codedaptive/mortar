//! A child command in the world. Real processes: the shell step (steps/shell.rs) runs git and mortar through it on
//! every drive.

use std::path::Path;
use std::process::{Command, Stdio};

use super::{world_env, World};

impl World {
    /// `program` in `cwd` with only the world's environment (the test's PATH, HOME and TMPDIR at the root, no
    /// global or system git config), stdin closed, stdout and stderr captured.
    pub fn command(&self, program: &Path, cwd: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .env_clear()
            .envs(world_env(&self.root, &std::env::var_os("PATH").unwrap_or_default()))
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
}
