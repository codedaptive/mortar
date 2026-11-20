//! choose_undo: which op of the log a folder's undo reverses, and what it does to the folder.
//! Tested by choose_undo_tests.rs, including a resolve step and an undo of a join-rebase, decided
//! through undo_reason's is_resolve_step and done_record.

use oplog::{OpPhase, OpRecord, ValuedRecord};

use super::{UndoChoice, UndoStep};
use crate::undo_reason::{done_record, is_resolve_step, reversed_op};

/// The last done record of `actor`, of a kind other than "snapshot" (a save's end-of-turn snapshot never hides the
/// save), with a move of any ref in `own` or of `zone_ref`. None is Nothing. One that moved `zone_ref` is ZoneOwns;
/// one that moved any ref not in `own` is NotAStep. Then by kind: "save" is a SetIndex step; "join-rebase" that
/// is_resolve_step (it records the paused join) is a RestoreJoin step; "join-rebase" with a "folder_tree" value and
/// no paused join is Resolved (a resolve recorded before mortar recorded the paused join); any other "join-rebase"
/// is a MoveFolder step; "undo" is AlreadyUndone with the step of the kind its reason names (save or join-rebase),
/// else NotAStep; any other kind is NotAStep.
pub fn choose_undo(records: &[ValuedRecord], actor: &str, own: &[String], zone_ref: &str) -> UndoChoice {
    let reaches = |refname: &String| own.contains(refname) || refname == zone_ref;
    let Some(index) = records.iter().rposition(|valued| {
        let record = &valued.record;
        record.phase == OpPhase::Done
            && record.actor == actor
            && record.kind != "snapshot"
            && record.moves.iter().any(|one| reaches(&one.refname))
    }) else {
        return UndoChoice::Nothing;
    };
    let chosen = &records[index];
    let ValuedRecord { record, values } = chosen;
    if record.moves.iter().any(|one| one.refname == zone_ref) {
        return UndoChoice::ZoneOwns { op_id: record.op_id.clone() };
    }
    if record.moves.iter().any(|one| !own.contains(&one.refname)) {
        return not_a_step(record);
    }
    match record.kind.as_str() {
        "save" => UndoChoice::Step { index, step: UndoStep::SetIndex },
        "join-rebase" if is_resolve_step(chosen) => UndoChoice::Step { index, step: UndoStep::RestoreJoin },
        "join-rebase" if values.iter().any(|(name, _)| name == "folder_tree") => {
            UndoChoice::Resolved { op_id: record.op_id.clone() }
        }
        "join-rebase" => UndoChoice::Step { index, step: UndoStep::MoveFolder },
        "undo" => already_undone(records, record),
        _ => not_a_step(record),
    }
}

/// AlreadyUndone for an undo whose reason names a save or a join-rebase; NotAStep for any other reason. A reversed
/// join-rebase whose Done record in `records` is_resolve_step gives RestoreJoin; any other, or one missing from the
/// log, gives MoveFolder.
fn already_undone(records: &[ValuedRecord], undo: &OpRecord) -> UndoChoice {
    let Some((kind, reversed)) = reversed_op(&undo.reason) else {
        return not_a_step(undo);
    };
    let step = match kind.as_str() {
        "save" => UndoStep::SetIndex,
        "join-rebase" if done_record(records, &reversed).is_some_and(is_resolve_step) => UndoStep::RestoreJoin,
        "join-rebase" => UndoStep::MoveFolder,
        _ => return not_a_step(undo),
    };
    UndoChoice::AlreadyUndone { undo_op: undo.op_id.clone(), reversed_op: reversed, step }
}

fn not_a_step(record: &OpRecord) -> UndoChoice {
    UndoChoice::NotAStep { op_id: record.op_id.clone(), kind: record.kind.clone() }
}
