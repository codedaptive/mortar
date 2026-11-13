//! usable_sparse_path: refuse a sparse path git could not be given.

use crate::no_nul::no_nul;
use crate::text_refusal::text_refusal;
use crate::GitError;

/// Ok for a non-empty path without a NUL or a line break, each refused naming the path. An empty path names no
/// folder; a NUL cannot pass through argv; git writes a line break raw into the folder's sparse-checkout file, which
/// then is not a cone pattern, so the folder silently leaves cone mode. A carriage return is stored and read back as
/// given.
pub(crate) fn usable_sparse_path(path: &str) -> Result<(), GitError> {
    if path.is_empty() {
        return Err(text_refusal("not a usable sparse path (empty)", path));
    }
    no_nul("sparse path", path.as_bytes())?;
    if path.contains('\n') {
        return Err(text_refusal("sparse path holds a line break", path));
    }
    Ok(())
}
