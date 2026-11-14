//! read_is_ancestor: read the answer of `merge-base --is-ancestor`.

use crate::{GitError, Output};

/// Exit 0 with nothing on stdout or stderr is true (an ancestor, or the same commit); exit 1 with nothing on either
/// is false. Anything else is refused naming the exit, stdout and stderr: `--is-ancestor` prints nothing when it
/// answers. An unknown commit is exit 128, which is not an answer code, so Git::run has already returned it as git's
/// error.
pub(crate) fn read_is_ancestor(output: &Output) -> Result<bool, GitError> {
    match (output.status, output.stdout.is_empty(), output.stderr.is_empty()) {
        (0, true, true) => Ok(true),
        (1, true, true) => Ok(false),
        (status, _, _) => Err(GitError::refusal(
            Vec::new(),
            format!(
                "merge-base --is-ancestor answered exit {status} with stdout {:?} and stderr {:?}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )),
    }
}
