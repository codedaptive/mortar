//! already_undone_report: the report of an undo whose step was already undone: a repeat, never a redo (S-142
//! item 3).

use super::UndoStep;
use crate::report::{ExitCode, OutValue, Report};

/// Done (exit 0), outcome "already-done", next mortar status. Lines: op `reversed_op` was already undone by op
/// `undo_op`; then the folder as the step leaves it: for SetIndex the files are untouched and the work shows as
/// unsaved, for MoveFolder the folder stands at the tip before the rebase. Fields: undone_op and undo_op.
pub fn already_undone_report(undo_op: &str, reversed_op: &str, step: UndoStep) -> Report {
    let step_line = match step {
        UndoStep::SetIndex => "the files are untouched; the work shows as unsaved",
        UndoStep::MoveFolder => "the folder stands at the tip before the rebase",
    };
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Done,
        outcome: "already-done".to_string(),
        lines: vec![format!("op {reversed_op} was already undone by op {undo_op}"), step_line.to_string()],
        owner: None,
        next: "mortar status".to_string(),
        fields: vec![
            ("undone_op".to_string(), OutValue::Text(reversed_op.to_string())),
            ("undo_op".to_string(), OutValue::Text(undo_op.to_string())),
        ],
    }
}
