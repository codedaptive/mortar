//! Git::new: the default git for one repository.

use crate::Git;
use std::path::PathBuf;

impl Git {
    /// Git for `repo` using `git` found on PATH, with no extra environment.
    pub fn new(repo: impl Into<PathBuf>) -> Git {
        Git {
            program: PathBuf::from("git"),
            repo: repo.into(),
            env: Vec::new(),
        }
    }
}
