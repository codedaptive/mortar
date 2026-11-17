//! nothing_to_undo_report: the report of an undo that found no step of the folder's own to undo (R7).

use crate::report::{ExitCode, Report};

/// Done (exit 0), outcome "nothing-to-undo", saying so, with `next`: what status gives for the folder.
pub fn nothing_to_undo_report(next: &str) -> Report {
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Done,
        outcome: "nothing-to-undo".to_string(),
        lines: vec!["nothing to undo: no save, rebase or resolve of this folder is in the op log".to_string()],
        owner: None,
        next: next.to_string(),
        fields: Vec::new(),
    }
}
