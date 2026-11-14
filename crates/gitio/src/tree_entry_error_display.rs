//! Display for TreeEntryError: the not-a-blob text naming the path and the type, or the GitError's own text.

use crate::{EntryKind, TreeEntryError};
use std::fmt;

impl fmt::Display for TreeEntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeEntryError::NotABlob { path, kind } => write!(f, "not a blob: {path:?} is a {:?}", name(kind)),
            TreeEntryError::Git(error) => fmt::Display::fmt(error, f),
        }
    }
}

/// The type as ls-tree names it.
fn name(kind: &EntryKind) -> &str {
    match kind {
        EntryKind::Commit => "commit",
        EntryKind::Tree => "tree",
        EntryKind::Other(name) => name,
    }
}
