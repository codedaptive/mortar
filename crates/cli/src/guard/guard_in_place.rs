//! Whether the guard as guard_status found it is the guard the config asks for.

use land::{GuardState, HookState, LandRefs};

/// True when both hooks are Mortar's own and guard.json names exactly `refs`; false when a hook is missing or
/// foreign, guard.json is missing, or it names other branches or another remote.
pub fn guard_in_place(state: &GuardState, refs: &LandRefs) -> bool {
    state.reference_transaction == HookState::Mortar
        && state.pre_push == HookState::Mortar
        && state.refs.as_ref() == Some(refs)
}
