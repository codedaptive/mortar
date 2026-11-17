//! HookState: what sits at one of the guard's hook paths. A bare type (S-7); classify_hook_tests exercises Missing.
//! No test of Mortar and Foreign is in land today: they are owed to Nagatha as
//! RITs (cases 1-7 and 9-11 in ledger/reports/WO-land-guard-symref-for-nagatha.md).

/// Missing: no file. Mortar: exactly the script the guard writes for that hook. Foreign: anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookState {
    Missing,
    Mortar,
    Foreign,
}
