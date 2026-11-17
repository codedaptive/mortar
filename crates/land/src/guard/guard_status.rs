//! guard_status: whether the guard is installed, read without writing. It runs git and reads files, so it has no
//! unit test (S-5). tests/land_rit/guard_installation.rs compares its whole GuardState after installs, for an
//! untouched repo, and for a foreign pre-push.

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
