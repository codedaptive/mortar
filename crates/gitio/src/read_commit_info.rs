//! read_commit_info: read the one record commit_info_request prints.

use crate::read_message::read_message;
use crate::{CommitInfo, GitError, Oid, Person};

/// The record is `<H>\0<T>\0<P>\0<an>\0<ae>\0<ad>\0<B>\0`: the last NUL is `-z`'s record end, the first six split
/// id, tree, parents, author name, author email and author date, and the rest is the message. The id must be
/// `commit`; parents are space-separated ids, none when empty. The author fields (date included) and the message are
/// UTF-8 text, kept unchanged (the date's form is not checked: git printed it). Empty output means `commit` names no
/// commit (git prints nothing for a tree). Anything else is refused naming the text found.
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
    let fields: Vec<&[u8]> = record.splitn(7, |b| *b == 0).collect();
    let [id, tree, parents, name, email, date, message] = fields[..] else {
        return Err(refuse("fewer than seven fields".into()));
    };
    if id != commit.as_str().as_bytes() {
        return Err(refuse(format!("id is not {commit}")));
    }
    Ok(CommitInfo {
        tree: Oid::parse(&String::from_utf8_lossy(tree))?,
        parents: read_parents(parents)?,
        author: Person {
            name: read_message(name)?,
            email: read_message(email)?,
            date: Some(read_message(date)?),
        },
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
