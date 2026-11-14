//! guard_status: whether the guard is installed, read without writing. It runs git and reads files, so it has no
//! unit test (S-5); tests/guard_zit.rs covers it: status_of_an_untouched_repo_and_of_a_foreign_hook,
//! fresh_install_writes_config_and_hooks, repeat_install_changes_nothing, install_follows_a_relative_hooks_path and
//! install_from_a_linked_worktree_covers_the_main_repo.

use gitio::Git;

use super::read_guard_config::read_guard_config;
use super::read_hooks::read_hooks;
use super::{GuardError, GuardState};

/// guard.json's branches and the state of both hooks in the folder git runs hooks from (core.hooksPath honoured).
pub fn guard_status(git: &Git) -> Result<GuardState, GuardError> {
    let refs = read_guard_config(git)?;
    let hooks_dir = git.hooks_path().map_err(GuardError::Git)?;
    let [(_, reference_transaction), (_, pre_push)] = read_hooks(&hooks_dir)?;
    Ok(GuardState { refs, hooks_dir, reference_transaction, pre_push })
}
