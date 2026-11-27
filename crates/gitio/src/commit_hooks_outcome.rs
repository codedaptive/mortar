//! CommitHooksOutcome: the answer of Git::commit_hooks. A declaration only; tests/typed/commit_hooks.rs exercises it.

use crate::HookRefusal;

/// What the commit hooks said about a commit about to be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommitHooksOutcome {
    /// Both hooks passed or were skipped: the message to commit, as the commit-msg hook left it.
    Message(String),
    /// A hook exited non-zero or was killed; the commit must not be made.
    Refused(HookRefusal),
}
