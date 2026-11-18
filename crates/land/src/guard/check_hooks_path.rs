//! check_hooks_path: read core.hooksPath from git and refuse a value git would resolve per worktree.
//! It runs git, so it has no sibling unit test (S-5); its decision is decide_hooks_path, with decide_hooks_path_tests.
//! A RIT is owed to Nagatha: on a throwaway repository, core.hooksPath unset and absolute pass, a relative value and
//! a `~/` value (which `--type=path` expands to an absolute one) give their outcomes, and the Git display arm reads
//! as git's failure (tests/land_rit/owed_displays.rs).

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
