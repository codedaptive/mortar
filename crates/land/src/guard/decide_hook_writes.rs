//! decide_hook_writes: which guard hooks guard_install writes and which it only sets the mode of, or the refusal.
//! decide_hook_writes_tests covers every case.

use std::path::PathBuf;

use super::guard_hook_names::GUARD_HOOK_NAMES;
use super::{GuardError, HookState};

/// `hooks` is each guard hook's path and state, in GUARD_HOOK_NAMES order. Any Foreign: Err(ForeignHook) naming the
/// first Foreign path, so nothing is written. Else one entry per hook, zipped with GUARD_HOOK_NAMES: Missing gives
/// (path, Some(name)), write that hook's script; Mortar gives (path, None), set its mode only. Foreign gives no
/// entry; it cannot occur after the refusal.
pub(crate) fn decide_hook_writes(
    hooks: &[(PathBuf, HookState)],
) -> Result<Vec<(PathBuf, Option<&'static str>)>, GuardError> {
    if let Some((path, _)) = hooks.iter().find(|(_, state)| *state == HookState::Foreign) {
        return Err(GuardError::ForeignHook { path: path.clone() });
    }
    Ok(hooks
        .iter()
        .zip(GUARD_HOOK_NAMES)
        .filter_map(|((path, state), name)| match state {
            HookState::Missing => Some((path.clone(), Some(name))),
            HookState::Mortar => Some((path.clone(), None)),
            HookState::Foreign => None,
        })
        .collect())
}
