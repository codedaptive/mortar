//! restore_failed_report: the report of `mr undo` when join's restore_join_state refused after the oplog undo had
//! already moved the ref (S-148, S-155 Q2).

use ::join::JoinRunError;

use super::restore_why;
use crate::report::{ExitCode, OutValue, Report};

/// Every error is exit 1 "refused": op `undone_op` was undone by op `undo_op` but the paused join was not put back,
/// then the why lines. The next step is mr undo, whose repeat runs the restore again and finishes it.
pub fn restore_failed_report(error: &JoinRunError, undone_op: &str, undo_op: &str) -> Report {
    let mut lines = vec![format!("op {undone_op} was undone by op {undo_op}; the paused join was not put back")];
    lines.extend(restore_why(error));
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: None,
        next: "mr undo".to_string(),
        fields: vec![
            ("undone_op".to_string(), OutValue::Text(undone_op.to_string())),
            ("undo_op".to_string(), OutValue::Text(undo_op.to_string())),
        ],
    }
}
