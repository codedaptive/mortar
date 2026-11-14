//! classify_hook: whether a hook file is absent, the guard's own, or someone else's.

use super::guard_hook_script::guard_hook_script;
use super::HookState;

/// `existing` is the hook file's bytes, None when there is no file. Only a byte-for-byte copy of the guard's script
/// for `name` is Mortar's, so any edit makes the hook Foreign and the guard leaves it alone.
pub(crate) fn classify_hook(name: &str, existing: Option<&[u8]>) -> HookState {
    match existing {
        None => HookState::Missing,
        Some(bytes) if bytes == guard_hook_script(name).as_bytes() => HookState::Mortar,
        Some(_) => HookState::Foreign,
    }
}
