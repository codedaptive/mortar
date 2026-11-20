//! How a guard failure reads: git's failure, the file and its I/O error, or the refusal naming its file.

use std::fmt;

use super::GuardError;

impl fmt::Display for GuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuardError::Git(e) => write!(f, "git failed: {e}"),
            GuardError::Io { path, error } => write!(f, "{}: {error}", path.display()),
            GuardError::ForeignHook { path } => write!(
                f,
                "{} is a hook Mortar did not write, and Mortar never overwrites one: move it aside or fold it in by \
                 hand, then install the guard again",
                path.display()
            ),
            GuardError::BadConfig { path, why } => {
                write!(f, "{} is not a Mortar guard config: {why}", path.display())
            }
        }
    }
}
