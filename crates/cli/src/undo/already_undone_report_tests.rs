//! already_undone_report: a repeat of an undo is exit 0 "already-done", naming the undo op and the op it undid,
//! never a redo; after a save's undo the files are untouched and the work shows as unsaved; after a
//! resolve's undo the files are untouched and the next is mr resolve. The --json fields are not pinned here.

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

#[test]
fn an_undone_rebase_says_the_folder_stands_at_the_tip_before_it() {
    let report = already_undone_report("op-13", "op-8", UndoStep::MoveFolder);
    assert_eq!(report.lines[1], "the folder stands at the tip before the rebase");
    assert_eq!(report.next, "mr status");
}

#[test]
fn an_undone_resolve_says_it_was_already_undone_and_sends_the_agent_to_resolve_again() {
    assert_eq!(
        shown(already_undone_report("op-14", "op-9", UndoStep::RestoreJoin)),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Done,
            outcome: "already-done".to_string(),
            lines: vec![
                "op op-9 was already undone by op op-14".to_string(),
                "the files are untouched; the branch stands at the tip before the resolve".to_string(),
            ],
            owner: None,
            next: "mr resolve".to_string(),
            fields: Vec::new(),
        }
    );
}
