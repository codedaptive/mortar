//! oid_line: read git output that is one object id and a newline.

use crate::{GitError, Oid};

/// Exactly `<oid>\n`, as rev-parse and commit-tree print it. Anything else is refused naming the text.
pub(crate) fn oid_line(stdout: &[u8]) -> Result<Oid, GitError> {
    let text = String::from_utf8_lossy(stdout);
    match text.strip_suffix('\n') {
        Some(hex) => Oid::parse(hex),
        None => Err(GitError::refusal(
            Vec::new(),
            format!("not one object id line (no final newline): {text:?}"),
        )),
    }
}
