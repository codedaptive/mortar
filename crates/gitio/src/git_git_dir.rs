//! Git::git_dir: a folder's own git directory. It only runs git, so it has no unit tests; Nagatha's integration test
//! "git_dir answering a linked worktree's own .git/worktrees/<name>, absolute" is to cover it.

use crate::git_dir_request::git_dir_request;
use crate::path_line::path_line;
use crate::{Git, GitError};
use std::path::{Path, PathBuf};

impl Git {
    /// The absolute path of `folder`'s git directory: the main worktree's `.git`, or a linked worktree's own
    /// `.git/worktrees/<name>`.
    pub fn git_dir(&self, folder: &Path) -> Result<PathBuf, GitError> {
        let (args, opts) = git_dir_request(folder)?;
        path_line(&self.run(&args, &opts)?.stdout)
    }
}
