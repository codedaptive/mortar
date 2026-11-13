//! How a Mortar trailer failure reads: git's failure, or the change ID refusal as it is.

use std::fmt;

use super::MortarTrailerError;

impl fmt::Display for MortarTrailerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MortarTrailerError::Git(e) => write!(f, "git trailers failed: {e}"),
            MortarTrailerError::ChangeId(e) => write!(f, "{e}"),
        }
    }
}
