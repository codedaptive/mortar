//! A Ran for a child that could not be started.

use std::io;

use super::Ran;

impl Ran {
    /// No exit and no signal, the error as its stderr, so a check line shows why.
    pub fn from_error(what: &str, error: io::Error) -> Ran {
        Ran { code: None, signal: None, stdout: String::new(), stderr: format!("could not run {what}: {error}") }
    }
}
