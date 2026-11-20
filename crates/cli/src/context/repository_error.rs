//! What a git run that asked for a directory's repository failed with.

use std::path::Path;

use gitio::GitError;

use super::ContextError;

/// Git ran and refused (it has an exit code): `dir` is in no repository, NotARepository. No exit code: git
/// itself failed (did not start, was killed), Git.
pub(crate) fn repository_error(dir: &Path, error: GitError) -> ContextError {
    match error.code {
        Some(_) => ContextError::NotARepository { dir: dir.to_path_buf(), error },
        None => ContextError::Git(error),
    }
}
