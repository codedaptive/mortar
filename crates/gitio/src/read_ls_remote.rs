//! read_ls_remote: read the answer of `git ls-remote`.

use crate::text_refusal::text_refusal;
use crate::{GitError, Oid};

/// Each `<oid>\t<refname>\n` line as (oid, refname), in git's order; empty output is no refs (git's exit 0 for
/// patterns matching nothing). Git matches each pattern against the tail of a refname by whole path parts, so `main`
/// and `heads/main` both list `refs/heads/main`. An annotated tag's peeled line `<commit oid>\trefs/tags/v1^{}` is
/// printed when a pattern matches that name too (no pattern, or a glob such as `tags/*`; the exact `v1` does not) and
/// is kept here as printed. Output without its final newline, or a line that is not an id, a tab and a non-empty
/// name, is refused naming the text or the line.
pub(crate) fn read_ls_remote(stdout: &[u8]) -> Result<Vec<(Oid, String)>, GitError> {
    let text = String::from_utf8_lossy(stdout);
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let body = text
        .strip_suffix('\n')
        .ok_or_else(|| text_refusal("ls-remote lines without their final newline", &text))?;
    body.split('\n').map(ref_line).collect()
}

/// One `<oid>\t<refname>` line; anything else is refused naming the line.
fn ref_line(line: &str) -> Result<(Oid, String), GitError> {
    line.split_once('\t')
        .filter(|(_, name)| !name.is_empty())
        .and_then(|(hex, name)| Some((Oid::parse(hex).ok()?, name.to_string())))
        .ok_or_else(|| text_refusal("not an ls-remote line", line))
}
