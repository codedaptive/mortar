//! TreeEntryError: why Git::tree_entry_typed found no blob id.

use crate::{EntryKind, GitError};
use std::path::PathBuf;

/// A refusal of Git::tree_entry_typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeEntryError {
    /// The entry at `path` is a folder, a gitlink or another non-blob type, named by `kind`.
    NotABlob { path: PathBuf, kind: EntryKind },
    /// Git's own failure, an empty or NUL-holding path, or an ls-tree answer of the wrong shape.
    Git(GitError),
}

impl std::error::Error for TreeEntryError {}
