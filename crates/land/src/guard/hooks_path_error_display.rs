//! How a hooks-path refusal reads: git's failure, or the relative value and why one install cannot guard it.

use std::fmt;

use super::HooksPathError;

impl fmt::Display for HooksPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HooksPathError::Git(e) => write!(f, "git could not read core.hooksPath: {e}"),
            HooksPathError::Relative { value } => write!(
                f,
                "core.hooksPath is \"{}\", not an absolute path: git resolves a relative hooks path in each folder, so \
                 one install cannot guard them all",
                value.display()
            ),
        }
    }
}
