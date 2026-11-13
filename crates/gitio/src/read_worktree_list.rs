//! read_worktree_list: read `git worktree list --porcelain -z`.

use crate::path_from_bytes::path_from_bytes;
use crate::read_message::read_message;
use crate::{GitError, Oid, Worktree};

/// One Worktree per record, in git's order; empty output is no worktrees. A record is NUL-ended fields closed by an
/// empty field, the first being `worktree <path>`. The attributes read are `HEAD <oid>`, `branch <ref>`, `bare`,
/// `detached`, `locked[ <reason>]` and `prunable[ <reason>]`; any other attribute is skipped, as newer gits may add
/// some. A record not starting with its path, a bad HEAD id, an empty or non-UTF-8 branch, or output not ended by
/// an empty field is refused naming the text.
pub(crate) fn read_worktree_list(stdout: &[u8]) -> Result<Vec<Worktree>, GitError> {
    let mut worktrees = Vec::new();
    let mut open: Option<Worktree> = None;
    let mut rest = stdout;
    while let Some(at) = rest.iter().position(|b| *b == 0) {
        let field = &rest[..at];
        rest = &rest[at + 1..];
        open = match open {
            None => Some(first_field(field)?),
            Some(worktree) if field.is_empty() => {
                worktrees.push(worktree);
                None
            }
            Some(mut worktree) => {
                attribute(&mut worktree, field)?;
                Some(worktree)
            }
        };
    }
    if open.is_some() || !rest.is_empty() {
        return Err(refuse("worktree list not ended by an empty field", stdout));
    }
    Ok(worktrees)
}

/// `worktree <path>`, the path non-empty.
fn first_field(field: &[u8]) -> Result<Worktree, GitError> {
    match field.strip_prefix(b"worktree ") {
        Some(path) if !path.is_empty() => Ok(Worktree {
            path: path_from_bytes(path),
            head: None,
            branch: None,
            bare: false,
            detached: false,
            locked: false,
            prunable: false,
        }),
        _ => Err(refuse("worktree record not starting with \"worktree <path>\"", field)),
    }
}

/// Sets what one attribute field says; the value is what follows the first space.
fn attribute(worktree: &mut Worktree, field: &[u8]) -> Result<(), GitError> {
    let (key, value) = match field.iter().position(|b| *b == b' ') {
        Some(at) => (&field[..at], &field[at + 1..]),
        None => (field, &b""[..]),
    };
    match key {
        b"HEAD" => worktree.head = Some(Oid::parse(&String::from_utf8_lossy(value))?),
        b"branch" if value.is_empty() => return Err(refuse("worktree branch line without a name", field)),
        b"branch" => worktree.branch = Some(read_message(value)?),
        b"bare" => worktree.bare = true,
        b"detached" => worktree.detached = true,
        b"locked" => worktree.locked = true,
        b"prunable" => worktree.prunable = true,
        _ => {}
    }
    Ok(())
}

fn refuse(what: &str, text: &[u8]) -> GitError {
    GitError::refusal(Vec::new(), format!("{what}: {:?}", String::from_utf8_lossy(text)))
}
