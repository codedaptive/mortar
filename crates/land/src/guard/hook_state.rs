//! HookState: what sits at one of the guard's hook paths. A bare type; classify_hook_tests exercises Missing.
//! tests/land_rit/guard_installation.rs asserts Missing, Mortar and Foreign through guard_status.

/// Missing: no file. Mortar: exactly the script the guard writes for that hook. Foreign: anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookState {
    Missing,
    Mortar,
    Foreign,
}
