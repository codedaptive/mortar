//! read_merge_base: read the answer of `merge-base`.

use crate::oid_line::oid_line;
use crate::{GitError, Oid, Output};

/// Exit 0 is the common ancestor's id line. Exit 1 with nothing on stdout or stderr is unrelated histories (None).
/// Anything else is refused naming the exit, stdout and stderr.
pub(crate) fn read_merge_base(output: &Output) -> Result<Option<Oid>, GitError> {
    match (output.status, output.stdout.is_empty(), output.stderr.is_empty()) {
        (0, _, _) => oid_line(&output.stdout).map(Some),
        (1, true, true) => Ok(None),
        (status, _, _) => Err(GitError::refusal(
            Vec::new(),
            format!(
                "merge-base answered exit {status} with stdout {:?} and stderr {:?}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )),
    }
}
