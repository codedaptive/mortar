//! Git::hook_folder: the folder git runs commit hooks in. It only runs git, so it has no unit tests;
//! tests/typed/commit_hooks.rs covers both branches: index_is_tree_from_subfolder (the worktree top from a subfolder)
//! and bare_repository_hooks_run_in_the_git_dir (a bare repository's git dir).

use crate::hook_folder_request::hook_folder_request;
use crate::read_bare_flag::read_bare_flag;
use crate::{Git, GitError};
use std::path::PathBuf;

impl Git {
    /// The folder git runs commit hooks in (githooks(5)): the top of the worktree `repo` is in, or for a bare
    /// repository its git dir.
    pub(crate) fn hook_folder(&self) -> Result<PathBuf, GitError> {
        let (args, opts) = hook_folder_request();
        if read_bare_flag(&self.run(&args, &opts)?.stdout)? {
            self.git_dir(&self.repo)
        } else {
            self.toplevel()
        }
    }
}
