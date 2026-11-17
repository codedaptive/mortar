//! GuardState: what guard_status found. A bare type (S-7). No test of its fields is in land today: they are owed to
//! Nagatha as RITs (cases 1, 2, 5-7 and 9-11 in ledger/reports/WO-land-guard-symref-for-nagatha.md).

use std::path::PathBuf;

use super::HookState;
use crate::LandRefs;

/// The branches guard.json names (None without one), the hooks folder git runs, and each hook's state there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardState {
    pub refs: Option<LandRefs>,
    pub hooks_dir: PathBuf,
    pub reference_transaction: HookState,
    pub pre_push: HookState,
}
