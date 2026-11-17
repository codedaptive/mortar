//! The two hooks the guard installs: reference-transaction stops local moves, pre-push stops pushes. A const only
//! (S-7). tests/land_rit/guard_codec.rs reaches it through guard_install but asserts no hook. Owed to Nagatha as
//! RITs, which assert both hook files by name (cases 1 and 5 in ledger/reports/WO-land-guard-symref-for-nagatha.md).

/// The hook file names, in the order decide_hook_writes checks them and names the first foreign one.
pub(crate) const GUARD_HOOK_NAMES: [&str; 2] = ["reference-transaction", "pre-push"];
