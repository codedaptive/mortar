//! already_undone_report: a repeat of an undo is exit 0 "already-done", naming the undo op and the op it undid,
//! never a redo (S-142 item 3); after a save's undo the files are untouched and the work shows as unsaved. The --json
//! fields are not pinned here.

use super::{already_undone_report, UndoStep};
use crate::report::{ExitCode, Report};

/// The report with its --json fields left out.
fn shown(report: Report) -> Report {
    Report { fields: Vec::new(), ..report }
}

#[test]
fn an_undone_save_says_it_was_already_undone_by_the_undo_op() {
    assert_eq!(
        shown(already_undone_report("op-12", "op-7", UndoStep::SetIndex)),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Done,
            outcome: "already-done".to_string(),
            lines: vec![
                "op op-7 was already undone by op op-12".to_string(),
                "the files are untouched; the work shows as unsaved".to_string(),
            ],
            owner: None,
            next: "mr status".to_string(),
            fields: Vec::new(),
        }
    );
}
