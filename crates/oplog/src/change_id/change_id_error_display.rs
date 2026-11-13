//! How a change ID refusal reads: the text and what is wrong with it, or the /dev/urandom failure.

use std::fmt;

use super::ChangeIdError;

impl fmt::Display for ChangeIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeIdError::Malformed { text, reason } => write!(f, "not a change ID: {text:?}: {reason}"),
            ChangeIdError::Random { source } => {
                write!(f, "cannot read 16 bytes from /dev/urandom for a new change ID: {source}")
            }
        }
    }
}
