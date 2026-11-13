//! read_commit_info: read the one record commit_info_request prints.

use crate::read_message::read_message;
use crate::{CommitInfo, GitError, Oid};

/// The record is `<H>\0<T>\0<P>\0<B>\0`: the last NUL is `-z`'s record end, the first three split id, tree and
/// parents, and the rest is the message (UTF-8, kept unchanged). The id must be `commit`; parents are space-separated
/// ids, none when empty. Empty output means `commit` names no commit (git prints nothing for a tree). Anything else
/// is refused naming the text found.
pub(crate) fn read_commit_info(stdout: &[u8], commit: &Oid) -> Result<CommitInfo, GitError> {
    if stdout.is_empty() {
        return Err(GitError::refusal(Vec::new(), format!("not a commit: {commit} (git printed nothing)")));
    }
    let refuse = |why: String| {
        GitError::refusal(
            Vec::new(),
            format!("not a commit record ({why}): {:?}", String::from_utf8_lossy(stdout)),
        )
    };
    let record = stdout.strip_suffix(b"\0").ok_or_else(|| refuse("no record NUL".into()))?;
    let mut fields = record.splitn(4, |b| *b == 0);
    let (Some(id), Some(tree), Some(parents), Some(message)) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(refuse("fewer than four fields".into()));
    };
    if id != commit.as_str().as_bytes() {
        return Err(refuse(format!("id is not {commit}")));
    }
    Ok(CommitInfo {
        tree: Oid::parse(&String::from_utf8_lossy(tree))?,
        parents: read_parents(parents)?,
        message: read_message(message)?,
    })
}

/// The `%P` field: empty for a root commit, else ids each followed by one space but the last.
fn read_parents(field: &[u8]) -> Result<Vec<Oid>, GitError> {
    if field.is_empty() {
        return Ok(Vec::new());
    }
    String::from_utf8_lossy(field).split(' ').map(Oid::parse).collect()
}
