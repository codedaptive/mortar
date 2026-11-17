//! A child command in the world. Real processes: the shell step (steps/shell.rs) runs git and `mr` through it on
//! every drive.

use std::path::Path;
use std::process::{Command, Stdio};

use super::{world_env, World};

impl World {
    /// `program` in `cwd` with only the world's environment (PATH the world's `path`, so every child, and every git
    /// hook that runs `mortar`, finds `mr` and `mortar` first; HOME and TMPDIR at the root; no global or system git
    /// config), stdin closed, stdout and stderr captured. A bare name such as `mr` is looked up on that PATH.
    pub fn command(&self, program: &Path, cwd: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .env_clear()
            .envs(world_env(&self.root, &self.path))
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
}
