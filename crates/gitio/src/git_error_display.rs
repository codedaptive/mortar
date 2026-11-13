//! Display for GitError: the command, how it ended, and what it said.

use crate::GitError;
use std::fmt;

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // A refusal made without running git has no args; it shows as "git: ...", not "git : ...".
        if self.args.is_empty() {
            f.write_str("git: ")?;
        } else {
            write!(f, "git {}: ", self.args.join(" "))?;
        }
        match self.code {
            Some(code) => write!(f, "exit code {code}")?,
            None => f.write_str("no exit code")?,
        }
        write!(f, ": {}", self.stderr.trim_end())
    }
}
