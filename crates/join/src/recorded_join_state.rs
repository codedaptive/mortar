//! recorded_join_state: the paused state a resolve's final join-rebase op recorded under JOIN_STATE (S-142(1)).
//! recorded_join_state_tests covers a good op, each cause of NotAResolveOp, the first of two values taken and each
//! cause of RecordedStateCorrupt.

use crate::decode_join_state::decode_join_state;
use crate::join_state::JoinState;
use crate::join_state_name::JOIN_STATE;
use crate::JoinRunError;
use oplog::{OpPhase, ValuedRecord};

/// The state `op` recorded, as mortar-join.json held it before the resolve's final run. NotAResolveOp unless the op
/// is a done join-rebase op with exactly one ref move and a JOIN_STATE value (the first one is taken). The value
/// must decode (decode_join_state) and match the op: rebased_to and folder_tree null, as join_state_value writes it,
/// request_ref the move's ref, and request_tip the move's old value; else RecordedStateCorrupt with the reason.
pub(crate) fn recorded_join_state(op: &ValuedRecord) -> Result<JoinState, JoinRunError> {
    let record = &op.record;
    let not_a_resolve_op = || JoinRunError::NotAResolveOp { op_id: record.op_id.clone() };
    let [ref_move] = record.moves.as_slice() else {
        return Err(not_a_resolve_op());
    };
    if record.kind != "join-rebase" || record.phase != OpPhase::Done {
        return Err(not_a_resolve_op());
    }
    let Some((_, text)) = op.values.iter().find(|(name, _)| name == JOIN_STATE) else {
        return Err(not_a_resolve_op());
    };
    let corrupt = |reason: String| JoinRunError::RecordedStateCorrupt { op_id: record.op_id.clone(), reason };
    let state = decode_join_state(text).map_err(corrupt)?;
    if state.rebased_to.is_some() || state.folder_tree.is_some() {
        let reason = "the recorded state has rebased_to or folder_tree set; a recorded state has both null";
        return Err(corrupt(reason.to_string()));
    }
    if state.request_ref != ref_move.refname {
        return Err(corrupt(format!(
            "the recorded state's request ref {} is not the op's moved ref {}",
            state.request_ref, ref_move.refname
        )));
    }
    if ref_move.old.as_ref() != Some(&state.request_tip) {
        return Err(corrupt(format!(
            "the op moved {} from {}, not from the recorded state's request tip {}",
            ref_move.refname,
            ref_move.old.as_ref().map_or("nothing".to_string(), ToString::to_string),
            state.request_tip
        )));
    }
    Ok(state)
}
