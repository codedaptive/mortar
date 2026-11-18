//! Reports for uninstall failures before and after record removal.

use std::path::PathBuf;
use crate::report::{ExitCode, Report};
use super::uninstall_failure;

#[test]
fn record_remaining_makes_repeat_possible() {
    assert_eq!(uninstall_failure("disk failed", &[]), Report {
        verb: "uninstall".into(), exit: ExitCode::Failure, outcome: "failed".into(),
        lines: vec!["disk failed".into(), "install's record is still in place, so a repeat finds what is left".into()],
        owner: None, next: "repeat the same command".into(), fields: vec![],
    });
}

#[test]
fn unrecorded_leftovers_are_named_in_order() {
    assert_eq!(uninstall_failure("disk failed", &[PathBuf::from("/h/a"), PathBuf::from("/h/b")]), Report {
        verb: "uninstall".into(), exit: ExitCode::Failure, outcome: "failed".into(),
        lines: vec!["disk failed".into(), "left, and no longer in install's record: /h/a".into(),
            "left, and no longer in install's record: /h/b".into()],
        owner: None, next: "remove each folder listed that is empty; a repeat no longer sees them".into(), fields: vec![],
    });
}
