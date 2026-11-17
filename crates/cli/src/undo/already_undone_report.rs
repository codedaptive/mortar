//! already_undone_report: the report of an undo whose step was already undone: a repeat, never a redo (S-142
//! item 3; S-148).

use super::UndoStep;
use crate::report::{ExitCode, OutValue, Report};

/// Done (exit 0), outcome "already-done". Lines: op `reversed_op` was already undone by op `undo_op`; then the folder
/// as the step leaves it: for SetIndex the files are untouched and the work shows as unsaved, for MoveFolder the
/// folder stands at the tip before the rebase, for RestoreJoin the files are untouched and the branch stands at the
/// tip before the resolve. Next is mr resolve for RestoreJoin (the paused join is back), else mr status. Fields:
/// undone_op and undo_op.
pub fn already_undone_report(undo_op: &str, reversed_op: &str, step: UndoStep) -> Report {
    let (step_line, next) = match step {
        UndoStep::SetIndex => ("the files are untouched; the work shows as unsaved", "mr status"),
        UndoStep::MoveFolder => ("the folder stands at the tip before the rebase", "mr status"),
        UndoStep::RestoreJoin => {
            ("the files are untouched; the branch stands at the tip before the resolve", "mr resolve")
        }
    };
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Done,
        outcome: "already-done".to_string(),
        lines: vec![format!("op {reversed_op} was already undone by op {undo_op}"), step_line.to_string()],
        owner: None,
        next: next.to_string(),
        fields: vec![
            ("undone_op".to_string(), OutValue::Text(reversed_op.to_string())),
            ("undo_op".to_string(), OutValue::Text(undo_op.to_string())),
        ],
    }
}
