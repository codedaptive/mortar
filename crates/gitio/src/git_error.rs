//! GitError: gitio's one error type.

/// A git run that failed, or a refusal made before or without running git.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitError {
    /// The git arguments, converted lossily: a non-UTF-8 argument shows with U+FFFD.
    pub args: Vec<String>,
    /// The exit code; None when git was killed by a signal, did not start, or never ran.
    pub code: Option<i32>,
    /// What git wrote to stderr (lossy UTF-8), or the refusal's message.
    pub stderr: String,
}

impl std::error::Error for GitError {}
