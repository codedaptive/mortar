//! read_oid_lines: read git output that is one object id per line, as rev-list prints it.

use crate::text_refusal::text_refusal;
use crate::{GitError, Oid};

/// Each `<oid>\n` line's id, in git's order; empty output is no ids. Output without its final newline, or with a
/// line that is empty or not an id, is refused naming the text.
pub(crate) fn read_oid_lines(stdout: &[u8]) -> Result<Vec<Oid>, GitError> {
    let text = String::from_utf8_lossy(stdout);
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let body = text.strip_suffix('\n').ok_or_else(|| text_refusal("id lines without their final newline", &text))?;
    body.split('\n').map(Oid::parse).collect()
}
