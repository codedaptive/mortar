//! candidate_size: the size the cap is checked against, from a changed path's metadata.

use std::io::{ErrorKind, Result};

/// `found` is (is_dir, len) from symlink_metadata, so a symlink is not followed. A file or symlink gives Some(len);
/// a directory (a gitlink or a plain one) gives None, as do NotFound (a deleted path) and NotADirectory (a path under
/// something that is a file, such as a deleted child of a directory replaced by a file): none of them is a file the
/// cap can refuse. Any other error passes through.
pub(crate) fn candidate_size(found: Result<(bool, u64)>) -> Result<Option<u64>> {
    match found {
        Ok((true, _)) => Ok(None),
        Ok((false, len)) => Ok(Some(len)),
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => Ok(None),
        Err(error) => Err(error),
    }
}
