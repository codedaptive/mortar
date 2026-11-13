//! check_exit: turn a finished run into an Output or a GitError.

use crate::{GitError, Output};

/// Ok when git exited 0 or with one of `answer_codes`; any other exit, or no exit code (killed by a signal), is Err.
pub(crate) fn check_exit(
    args: &[String],
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    answer_codes: &[i32],
) -> Result<Output, GitError> {
    match code {
        Some(status) if status == 0 || answer_codes.contains(&status) => Ok(Output {
            status,
            stdout,
            stderr,
        }),
        _ => Err(GitError {
            args: args.to_vec(),
            code,
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        }),
    }
}
