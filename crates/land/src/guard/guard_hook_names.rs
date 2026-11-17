//! The two hooks the guard installs: reference-transaction stops local moves, pre-push stops pushes. A const only
//! (S-7). tests/land_rit/guard_installation.rs asserts both hook files by name.

/// The hook file names, in the order decide_hook_writes checks them and names the first foreign one.
pub(crate) const GUARD_HOOK_NAMES: [&str; 2] = ["reference-transaction", "pre-push"];
