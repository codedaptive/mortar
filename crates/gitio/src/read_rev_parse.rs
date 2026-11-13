//! read_rev_parse: read the answer of `rev-parse --verify -q`.

use crate::oid_line::oid_line;
use crate::{GitError, Oid, Output};

/// Exit 0 is the commit's id line; exit 1 with nothing on stdout is "no such commit" (None). Anything else is
/// refused naming the exit and stdout.
pub(crate) fn read_rev_parse(output: &Output) -> Result<Option<Oid>, GitError> {
    match (output.status, output.stdout.is_empty()) {
        (0, _) => oid_line(&output.stdout).map(Some),
        (1, true) => Ok(None),
        (status, _) => Err(GitError::refusal(
            Vec::new(),
            format!(
                "rev-parse answered exit {status} with stdout {:?}",
                String::from_utf8_lossy(&output.stdout)
            ),
        )),
    }
}
