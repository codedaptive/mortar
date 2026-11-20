//! read_folder_move: read the answer of the two-tree `read-tree -m -u <old> <new>`, both the check and the move.
//! A refusal's path may hold quotes and line breaks, so stderr is read record by record, never by line: each record
//! runs from its head to the first following occurrence of its own tail. A path that itself holds its record's tail
//! text is cut short there; the bytes after it are usually no record, so the whole stderr is read as git's error, but
//! if they start with a record head they are read as further records naming paths that do not exist.

use crate::path_from_bytes::path_from_bytes;
use crate::{FolderMoveOutcome, GitError, Output};
use std::path::PathBuf;

const ENTRY: &[u8] = b"error: Entry '";
const ENTRY_TAILS: [&[u8]; 2] = [
    b"' not uptodate. Cannot merge.\n",
    b"' would be overwritten by merge. Cannot merge.\n",
];
const UNTRACKED: &[u8] = b"error: Untracked working tree file '";
const UNTRACKED_TAILS: [&[u8]; 1] = [b"' would be overwritten by merge.\n"];

/// Exit 0 with nothing on stdout is Moved. Exit 128 with nothing on stdout and stderr made wholly of git's refusal
/// records (a local edit, a staged change or an untracked, not ignored file in the way) is Refused, with the paths
/// in order. Any other exit 128 is git's own error (an unmerged index, an unknown tree), returned with its stderr.
/// Anything else is refused naming what was found.
pub(crate) fn read_folder_move(output: &Output) -> Result<FolderMoveOutcome, GitError> {
    match output.status {
        0 if output.stdout.is_empty() => Ok(FolderMoveOutcome::Moved),
        128 if output.stdout.is_empty() => match refused_paths(&output.stderr) {
            Some(paths) => Ok(FolderMoveOutcome::Refused(paths)),
            None => Err(gits_error(output)),
        },
        128 => Err(gits_error(output)),
        status => Err(GitError::refusal(
            Vec::new(),
            format!(
                "read-tree answered exit {status} with stdout {:?}",
                String::from_utf8_lossy(&output.stdout)
            ),
        )),
    }
}

/// Git's exit 128 with its stderr.
fn gits_error(output: &Output) -> GitError {
    GitError {
        args: Vec::new(),
        code: Some(128),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// The path of each refusal record, when stderr is one or more of them and nothing else.
fn refused_paths(mut rest: &[u8]) -> Option<Vec<PathBuf>> {
    let mut paths = Vec::new();
    while !rest.is_empty() {
        let (path, after) = next_record(rest)?;
        paths.push(path_from_bytes(path));
        rest = after;
    }
    (!paths.is_empty()).then_some(paths)
}

/// The path of the record `rest` starts with, and the bytes after that record.
fn next_record(rest: &[u8]) -> Option<(&[u8], &[u8])> {
    if let Some(body) = rest.strip_prefix(ENTRY) {
        return split_at_tail(body, &ENTRY_TAILS);
    }
    split_at_tail(rest.strip_prefix(UNTRACKED)?, &UNTRACKED_TAILS)
}

/// `body` split at the earliest occurrence of any of `tails`: the bytes before it, and those after it.
fn split_at_tail<'a>(body: &'a [u8], tails: &[&[u8]]) -> Option<(&'a [u8], &'a [u8])> {
    tails
        .iter()
        .filter_map(|tail| body.windows(tail.len()).position(|w| w == *tail).map(|at| (at, tail.len())))
        .min()
        .map(|(at, len)| (&body[..at], &body[at + len..]))
}
