//! guard_install: put the guard's config and both hooks in place. It runs git and does file I/O, so it has no sibling
//! unit test (S-5); its decisions are decide_hook_writes and decide_config_write, each with its _tests.
//! tests/guard_zit.rs covers it: fresh_install_writes_config_and_hooks, repeat_install_changes_nothing,
//! a_foreign_pre_push_is_refused_and_nothing_written, a_foreign_reference_transaction_is_refused_and_nothing_written,
//! install_follows_a_relative_hooks_path and install_from_a_linked_worktree_covers_the_main_repo.

use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use gitio::Git;

use super::decide_config_write::decide_config_write;
use super::decide_hook_writes::decide_hook_writes;
use super::guard_config_path::guard_config_path;
use super::guard_config_text::guard_config_text;
use super::guard_hook_script::guard_hook_script;
use super::read_hooks::read_hooks;
use super::read_optional::read_optional;
use super::write_replacing::write_replacing;
use super::GuardError;
use crate::LandRefs;

/// Hooks must run, so they are owner-writable and executable by all.
const HOOK_MODE: u32 = 0o755;

/// Installs the guard for `refs`: guard.json under the common dir, and the reference-transaction and pre-push hooks
/// in the folder git runs hooks from (core.hooksPath honoured; linked worktrees share it, so every folder is
/// covered). A hook Mortar did not write is refused as ForeignHook, the first in GUARD_HOOK_NAMES order, before
/// anything is written. guard.json is rewritten only when its bytes differ, and written before the hooks so a hook
/// never runs without its config. Missing hooks are written; Mortar's own get their mode set and are otherwise left.
/// Repeat-safe.
pub fn guard_install(git: &Git, refs: &LandRefs) -> Result<(), GuardError> {
    let hooks_dir = git.hooks_path().map_err(GuardError::Git)?;
    let common = git.common_dir().map_err(GuardError::Git)?;
    let hooks = read_hooks(&hooks_dir)?;
    let writes = decide_hook_writes(&hooks)?;
    let config = guard_config_path(&common);
    let text = guard_config_text(refs);
    if decide_config_write(read_optional(&config)?.as_deref(), &text) {
        create_dir_all(&common.join("mortar"))?;
        write_replacing(&config, text.as_bytes(), None)?;
    }
    // core.hooksPath may name a folder that does not exist yet.
    create_dir_all(&hooks_dir)?;
    for (path, name) in writes {
        match name {
            Some(name) => write_replacing(&path, guard_hook_script(name).as_bytes(), Some(HOOK_MODE))?,
            None => fs::set_permissions(&path, Permissions::from_mode(HOOK_MODE))
                .map_err(|error| GuardError::Io { path, error })?,
        }
    }
    Ok(())
}

/// fs::create_dir_all, failing as Io naming the folder.
fn create_dir_all(dir: &Path) -> Result<(), GuardError> {
    fs::create_dir_all(dir).map_err(|error| GuardError::Io { path: dir.to_path_buf(), error })
}
