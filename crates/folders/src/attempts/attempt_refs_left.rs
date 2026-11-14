//! The refusal keep and drop_attempts return when their op found a ref moved after they removed attempt folders.

use std::path::PathBuf;

use crate::make::FolderError;

/// RefMoved { refname, at } becomes AttemptRefsLeft naming `refname` as the moved ref, `at`, and the attempt folders
/// at `removed` already taken away; any other error is returned unchanged.
pub(crate) fn attempt_refs_left(err: FolderError, removed: &[PathBuf]) -> FolderError {
    match err {
        FolderError::RefMoved { refname, at } => FolderError::AttemptRefsLeft {
            moved: refname,
            at,
            removed: removed.to_vec(),
        },
        other => other,
    }
}
