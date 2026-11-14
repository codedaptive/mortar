//! Why the current directory's Mortar folder cannot be read.
//! A bare type (S-7); context_report_tests exercises each variant.

use std::path::PathBuf;

use folders::FolderError;
use gitio::GitError;

/// NotARepository: git ran and said `dir` is in no repository. NotAFolder: a worktree with no Mortar
/// record. Git: git failed some other way (did not start, was killed). Record: the record could not be read
/// (FolderError::Io) or is not one Mortar writes (FolderError::Record). Debug only: FolderError is neither Clone
/// nor Eq.
#[derive(Debug)]
pub enum ContextError {
    NotARepository { dir: PathBuf, error: GitError },
    NotAFolder { toplevel: PathBuf },
    Git(GitError),
    Record(FolderError),
}
