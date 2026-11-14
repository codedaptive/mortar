//! candidate_size: the size the cap is checked against, from a changed path's metadata.

use std::io::{ErrorKind, Result};

/// `found` is (is_dir, len) from symlink_metadata, so a symlink is not followed. A file or symlink gives Some(len);
/// a directory (a gitlink) gives None, as does NotFound (a deleted path): neither is a file the cap can refuse. Any
/// other error passes through.
pub(crate) fn candidate_size(found: Result<(bool, u64)>) -> Result<Option<u64>> {
    match found {
        Ok((true, _)) => Ok(None),
        Ok((false, len)) => Ok(Some(len)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
