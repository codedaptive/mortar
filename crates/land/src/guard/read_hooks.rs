//! read_hooks: each guard hook's path and state in one hooks folder. It reads files, so it has no unit test.
//! tests/land_rit/guard_installation.rs asserts missing, Mortar's and foreign hooks through guard_status and
//! guard_install.

use std::path::{Path, PathBuf};

use super::classify_hook::classify_hook;
use super::guard_hook_names::GUARD_HOOK_NAMES;
use super::read_optional::read_optional;
use super::{GuardError, HookState};

/// For each name in GUARD_HOOK_NAMES, in that order: its path under `hooks_dir` and what is there.
pub(crate) fn read_hooks(hooks_dir: &Path) -> Result<[(PathBuf, HookState); 2], GuardError> {
    let read = |name: &str| -> Result<(PathBuf, HookState), GuardError> {
        let path = hooks_dir.join(name);
        let state = classify_hook(name, read_optional(&path)?.as_deref());
        Ok((path, state))
    };
    Ok([read(GUARD_HOOK_NAMES[0])?, read(GUARD_HOOK_NAMES[1])?])
}
