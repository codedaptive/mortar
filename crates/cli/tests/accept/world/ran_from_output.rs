//! A Ran from a finished child. Real processes: the shell step (steps/shell.rs) reads every git and mortar child
//! through it on every drive.

use std::os::unix::process::ExitStatusExt;
use std::process::Output;

use super::Ran;

impl Ran {
    pub fn from_output(output: Output) -> Ran {
        Ran {
            code: output.status.code(),
            signal: output.status.signal(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}
