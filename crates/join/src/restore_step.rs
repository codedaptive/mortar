//! RestoreStep: what restore_join_state does with what it read. A declaration only; decide_restore_tests,
//! restore_state_step_tests and restore_refusal_tests exercise it.

use gitio::Oid;

/// The one step a restore takes, chosen by decide_restore and restore_state_step (restore_state_step never chooses
/// Stale).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RestoreStep {
    /// Write the recorded state as mortar-join.json.
    Write,
    /// mortar-join.json already holds the recorded state: a repeat, nothing to write.
    AlreadyRestored,
    /// The request ref is not at the recorded state's request tip (`actual` is None when the ref is gone).
    Stale { expected: Oid, actual: Option<Oid> },
    /// mortar-join.json holds a different paused join, which a restore never overwrites.
    StatePresent,
}
