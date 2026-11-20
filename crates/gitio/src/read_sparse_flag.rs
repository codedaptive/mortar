//! read_sparse_flag: read the answer of `config --type=bool --get core.sparseCheckout`.

use crate::{GitError, Output};

/// Whether the folder is sparse. Exit 1 is the key unset, so not sparse, whatever stdout holds. Exit 0 with stdout
/// exactly "true\n" is sparse and "false\n" is not (`--type=bool` prints every spelling of a bool as one of these).
/// Any other stdout is refused naming it.
pub(crate) fn read_sparse_flag(output: &Output) -> Result<bool, GitError> {
    if output.status == 1 {
        return Ok(false);
    }
    match output.stdout.as_slice() {
        b"true\n" => Ok(true),
        b"false\n" => Ok(false),
        other => Err(GitError::refusal(
            Vec::new(),
            format!("not a core.sparseCheckout bool: {:?}", String::from_utf8_lossy(other)),
        )),
    }
}
