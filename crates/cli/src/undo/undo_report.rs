//! undo_report: the report of an undo oplog answered Done, after its folder step (S-142, S-143, S-148).

use gitio::Oid;
use oplog::OpRecord;

use super::UndoStep;
use crate::report::{ExitCode, OutValue, Report};

/// Done (exit 0), outcome "undone". Lines: the kind and id of the op `undone`; the undo's own op `undo_op`; the
/// snapshot commit that keeps the folder as it stood before the undo; then the folder step's: for SetIndex the files
/// are untouched and the work shows as unsaved, for MoveFolder the folder is back at the tip before the rebase, for
/// RestoreJoin the files are untouched and the branch is back at the tip before the resolve. Next is mr resolve for
/// RestoreJoin (the paused join is back), else mr status. Fields: undone_op, undone_kind, undo_op and snapshot.
pub fn undo_report(undone: &OpRecord, undo_op: &str, snapshot: &Oid, step: UndoStep) -> Report {
    let (step_line, next) = match step {
        UndoStep::SetIndex => ("the files are untouched; the work shows as unsaved", "mr status"),
        UndoStep::MoveFolder => ("the folder is moved back to the tip before the rebase", "mr status"),
        UndoStep::RestoreJoin => {
            ("the files are untouched; the branch is back at the tip before the resolve", "mr resolve")
        }
    };
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Done,
        outcome: "undone".to_string(),
        lines: vec![
            format!("undid {} op {}", undone.kind, undone.op_id),
            format!("the undo is op {undo_op}"),
            format!("the folder as it stood is kept in snapshot commit {}", snapshot.as_str()),
            step_line.to_string(),
        ],
        owner: None,
        next: next.to_string(),
        fields: vec![
            ("undone_op".to_string(), OutValue::Text(undone.op_id.clone())),
            ("undone_kind".to_string(), OutValue::Text(undone.kind.clone())),
            ("undo_op".to_string(), OutValue::Text(undo_op.to_string())),
            ("snapshot".to_string(), OutValue::Text(snapshot.as_str().to_string())),
        ],
    }
}
