//! spawn_refusal: the error for a git process that could not be started or waited on.

use crate::GitError;
use std::path::Path;

/// A refusal (no exit code) naming the program and the io error.
pub(crate) fn spawn_refusal(program: &Path, args: &[String], err: &std::io::Error) -> GitError {
    GitError::refusal(
        args.to_vec(),
        format!("could not run {}: {err}", program.display()),
    )
}
