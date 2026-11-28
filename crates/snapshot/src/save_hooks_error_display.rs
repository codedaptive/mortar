//! Display for SaveHooksError: SnapshotError's own sentence, or one naming the refusing hook. The hook's stdout and
//! stderr are for the caller to show. save_hooks_error_display_tests.rs checks every variant's text.

use crate::hook_refusal_text::hook_refusal_text;
use crate::SaveHooksError;
use std::fmt;

impl fmt::Display for SaveHooksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveHooksError::Snapshot(error) => write!(f, "{error}"),
            SaveHooksError::HookRefused(refusal) => {
                write!(f, "{}; no commit was written and no branch moved", hook_refusal_text(refusal))
            }
        }
    }
}
