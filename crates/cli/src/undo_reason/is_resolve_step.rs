//! is_resolve_step: whether an op record is a resolve's final join step, the join-rebase op that records the paused
//! join state under JOIN_STATE beside folder_tree (S-142 (1)). An undo of such a step puts the paused join back and
//! leaves the folder's files, so it is no pending folder move (S-143, S-148).

use ::join::JOIN_STATE;
use oplog::ValuedRecord;

/// True when `valued` is a "join-rebase" op with a value named JOIN_STATE, whatever its text. A plain join rebase,
/// and a resolve written before JOIN_STATE was recorded, carry folder_tree only and are false.
pub fn is_resolve_step(valued: &ValuedRecord) -> bool {
    valued.record.kind == "join-rebase" && valued.values.iter().any(|(name, _)| name == JOIN_STATE)
}
