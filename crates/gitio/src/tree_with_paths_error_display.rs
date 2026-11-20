//! Display for TreeWithPathsError: the refusal's reason, or the git error as GitError shows it.
//! Its tests are ZUTs in tree_with_paths_error_display_tests.rs: Display only formats, so they use only pure code
//! of the kit.

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
