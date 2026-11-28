//! Display for CommitPathsHooksError: CommitPathsError's own sentence, or one naming the refusing hook. The hook's
//! stdout and stderr are for the caller to show. commit_paths_hooks_error_display_tests.rs checks every variant's
//! text.

use crate::hook_refusal_text::hook_refusal_text;
use crate::CommitPathsHooksError;
use std::fmt;

impl fmt::Display for CommitPathsHooksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitPathsHooksError::Paths(error) => write!(f, "{error}"),
            CommitPathsHooksError::HookRefused(refusal) => {
                write!(f, "{}; no commit was written", hook_refusal_text(refusal))
            }
        }
    }
}
