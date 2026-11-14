//! Display for IndexTreeError: the unmerged text naming each path, or the GitError's own text.

use crate::IndexTreeError;
use std::fmt;

impl fmt::Display for IndexTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexTreeError::Unmerged { paths } => {
                f.write_str("index has unmerged paths: ")?;
                for (at, path) in paths.iter().enumerate() {
                    if at > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{path:?}")?;
                }
                Ok(())
            }
            IndexTreeError::Git(error) => fmt::Display::fmt(error, f),
        }
    }
}
