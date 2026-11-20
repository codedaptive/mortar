//! read_merge_tree: read the answer of `merge-tree --write-tree -z --name-only`.

use crate::path_from_bytes::path_from_bytes;
use crate::{GitError, MergeTreeOutcome, Oid, Output};
use std::path::PathBuf;

/// stdout starts with the tree id as a NUL-ended field. Exit 0 is Clean. Exit 1 is Conflicted, with the file names
/// that follow up to the first empty field (the messages section after it is not read). Exit 1 with no tree id is
/// git refusing to merge (a bad or unknown rev): returned as git's error with its stderr, never as a conflict.
/// Anything else is refused naming what was found.
pub(crate) fn read_merge_tree(output: &Output) -> Result<MergeTreeOutcome, GitError> {
    let split = output.stdout.iter().position(|b| *b == 0);
    let tree = split.and_then(|at| Oid::parse(&String::from_utf8_lossy(&output.stdout[..at])).ok());
    match (output.status, tree, split) {
        (0, Some(tree), _) => Ok(MergeTreeOutcome::Clean(tree)),
        (1, Some(tree), Some(at)) => Ok(MergeTreeOutcome::Conflicted {
            tree,
            files: conflicted_files(&output.stdout[at + 1..])?,
        }),
        (1, None, _) => Err(GitError {
            args: Vec::new(),
            code: Some(1),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }),
        (status, _, _) => Err(GitError::refusal(
            Vec::new(),
            format!(
                "merge-tree answered exit {status} with stdout {:?}",
                String::from_utf8_lossy(&output.stdout)
            ),
        )),
    }
}

/// The NUL-ended names before the first empty field; at least one, and the empty field must be there.
fn conflicted_files(mut rest: &[u8]) -> Result<Vec<PathBuf>, GitError> {
    let mut files = Vec::new();
    while let Some(at) = rest.iter().position(|b| *b == 0) {
        if at == 0 {
            if files.is_empty() {
                break;
            }
            return Ok(files);
        }
        files.push(path_from_bytes(&rest[..at]));
        rest = &rest[at + 1..];
    }
    Err(GitError::refusal(
        Vec::new(),
        format!(
            "merge-tree conflict without a NUL-ended list of file names: {:?}",
            String::from_utf8_lossy(rest)
        ),
    ))
}
