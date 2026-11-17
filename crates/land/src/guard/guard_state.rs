//! GuardState: what guard_status found. A bare type (S-7). tests/land_rit/guard_installation.rs compares whole values
//! of it from guard_status: installed, untouched and with a foreign pre-push.

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
