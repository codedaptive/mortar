//! read_ref_list: read for-each-ref's `<oid> <refname>` lines.

use crate::read_message::read_message;
use crate::text_refusal::text_refusal;
use crate::{GitError, Oid};

/// One (id, refname) per `<oid> <refname>\n` line, in git's order; empty output is no refs. Output that is not
/// UTF-8, lacks its final newline, or has a line without a space, a bad id or an empty refname is refused.
pub(crate) fn read_ref_list(stdout: &[u8]) -> Result<Vec<(Oid, String)>, GitError> {
    let text = read_message(stdout)?;
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let body = text.strip_suffix('\n').ok_or_else(|| text_refusal("ref list without its final newline", &text))?;
    body.split('\n').map(ref_line).collect()
}

/// One line, its newline already removed.
fn ref_line(line: &str) -> Result<(Oid, String), GitError> {
    let (hex, refname) = line.split_once(' ').ok_or_else(|| text_refusal("ref line without a space", line))?;
    if refname.is_empty() {
        return Err(text_refusal("ref line with an empty refname", line));
    }
    Ok((Oid::parse(hex)?, refname.to_string()))
}
