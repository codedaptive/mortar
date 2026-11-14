//! HookState: what sits at one of the guard's hook paths. A bare type (S-7); classify_hook_tests exercises Missing and
//! tests/guard_zit Mortar and Foreign.

/// Missing: no file. Mortar: exactly the script the guard writes for that hook. Foreign: anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookState {
    Missing,
    Mortar,
    Foreign,
}
