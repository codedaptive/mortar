//! Display for TreeWithPathsError: the refusal's reason, or the git error as GitError shows it.
//! Its tests are ZITs in tests/display/tree_with_paths_error.rs, because both cases run another file's Display (S-104).

use crate::TreeWithPathsError;
use std::fmt;

impl fmt::Display for TreeWithPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeWithPathsError::Refused(refusal) => fmt::Display::fmt(refusal, f),
            TreeWithPathsError::Git(error) => fmt::Display::fmt(error, f),
        }
    }
}
