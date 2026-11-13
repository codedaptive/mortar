//! read_check_ref_format: read the answer of `check-ref-format`.

use crate::{GitError, Output};

/// Exit 0 with nothing on stdout is a well-formed name (true); exit 1 with nothing on stdout or stderr is a
/// malformed one (false). Anything else (a usage error printed on stderr, for one) is refused naming the exit,
/// stdout and stderr.
pub(crate) fn read_check_ref_format(output: &Output) -> Result<bool, GitError> {
    match (output.status, output.stdout.is_empty(), output.stderr.is_empty()) {
        (0, true, _) => Ok(true),
        (1, true, true) => Ok(false),
        (status, _, _) => Err(GitError::refusal(
            Vec::new(),
            format!(
                "check-ref-format answered exit {status} with stdout {:?} and stderr {:?}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )),
    }
}
