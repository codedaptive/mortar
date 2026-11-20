//! check_hooks_path: read core.hooksPath from git and refuse a value git would resolve per worktree.
//! It runs git, so it has no sibling unit test; its decision is decide_hooks_path, with decide_hooks_path_tests.
//! tests/land_rit/check_hooks_path.rs runs it on throwaway repositories: unset, absolute and `~/` values pass,
//! relative and empty values are refused, a broken config is a Git failure, and none of them writes anything.

use gitio::{Git, RunOpts};

use super::decide_hooks_path::decide_hooks_path;
use super::HooksPathError;

/// Refuses a core.hooksPath that git would resolve per worktree. Callers must run it before guard_install, which would
/// otherwise put hooks only where git looks from its own worktree.
pub fn check_hooks_path(git: &Git) -> Result<(), HooksPathError> {
    // Exit 1 from `git config --get` is an answer: the key is unset.
    let opts = RunOpts { answer_codes: vec![1], read_only: true, ..RunOpts::default() };
    let out = git
        .run(&["config", "--type=path", "--get", "core.hooksPath"], &opts)
        .map_err(HooksPathError::Git)?;
    if out.status == 1 {
        return decide_hooks_path(None);
    }
    decide_hooks_path(Some(&out.stdout))
}
