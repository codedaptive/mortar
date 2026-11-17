//! nothing_to_undo_report: exit 0 "nothing-to-undo", saying so, with the next step it is given (what status would
//! give for this folder, R7). The --json fields are not pinned here.

use super::nothing_to_undo_report;
use crate::report::{ExitCode, Report};

/// The report with its --json fields left out.
fn shown(report: Report) -> Report {
    Report { fields: Vec::new(), ..report }
}

#[test]
fn nothing_to_undo_says_so_with_the_next_given() {
    assert_eq!(
        shown(nothing_to_undo_report("mortar save \"<one line saying what changed>\"")),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-undo".to_string(),
            lines: vec!["nothing to undo: no save, rebase or resolve of this folder is in the op log".to_string()],
            owner: None,
            next: "mortar save \"<one line saying what changed>\"".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn the_next_is_the_one_passed_in() {
    for next in ["mortar join", "mortar status", "wait for the zone's coordinator"] {
        assert_eq!(nothing_to_undo_report(next).next, next);
    }
}
