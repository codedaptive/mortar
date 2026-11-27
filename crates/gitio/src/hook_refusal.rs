//! HookRefusal: a commit hook that refused the commit. A declaration only; tests/typed/commit_hooks.rs exercises it.

/// The hook that refused and what it wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookRefusal {
    /// The hook's name: "pre-commit" or "commit-msg".
    pub hook: &'static str,
    /// The hook's exit code; None when a signal killed it.
    pub code: Option<i32>,
    /// Everything the hook wrote to stdout.
    pub stdout: Vec<u8>,
    /// Everything the hook wrote to stderr, kept apart from stdout.
    pub stderr: Vec<u8>,
}
