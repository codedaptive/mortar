//! Reports for a failed install whose record was written.

use std::path::PathBuf;
use crate::report::{ExitCode, Report};
use super::{apply_failure, RecordItem};

#[test]
fn empty_record_says_nothing_was_applied() {
    assert_eq!(apply_failure("disk failed", &[], "mr"), Report {
        verb: "install".into(), exit: ExitCode::Failure, outcome: "failed".into(),
        lines: vec!["disk failed".into(), "no step was applied and no folder was made".into()],
        owner: None, next: "repeat the same command".into(), fields: vec![],
    });
}

#[test]
fn recorded_items_are_named_in_order() {
    let items = [
        RecordItem::File { path: PathBuf::from("/h/bin/mortar"), hash: "hash".into() },
        RecordItem::Link { path: PathBuf::from("/h/bin/mr"), target: PathBuf::from("/h/bin/mortar") },
        RecordItem::Dir { path: PathBuf::from("/h/bin") },
    ];
    assert_eq!(apply_failure("disk failed", &items, "mr"), Report {
        verb: "install".into(), exit: ExitCode::Failure, outcome: "failed".into(),
        lines: vec!["disk failed".into(),
            "recorded, so mr uninstall can remove it: /h/bin/mortar".into(),
            "recorded, so mr uninstall can remove it: /h/bin/mr".into(),
            "recorded, so mr uninstall can remove it: /h/bin".into()],
        owner: None, next: "once the failure above is fixed, repeat the same command to finish the install, or run mr uninstall to remove what was placed".into(), fields: vec![],
    });
}

#[test]
fn planned_mr_without_applied_link_uses_mortar_for_recovery() {
    let items = [RecordItem::File { path: PathBuf::from("/h/bin/mortar"), hash: "hash".into() }];
    assert_eq!(apply_failure("disk failed", &items, "mr"), Report {
        verb: "install".into(), exit: ExitCode::Failure, outcome: "failed".into(),
        lines: vec!["disk failed".into(),
            "recorded, so mortar uninstall can remove it: /h/bin/mortar".into()],
        owner: None, next: "once the failure above is fixed, repeat the same command to finish the install, or run mortar uninstall to remove what was placed".into(), fields: vec![],
    });
}

#[test]
fn selected_mortar_with_applied_link_uses_mortar_for_recovery() {
    let items = [RecordItem::Link {
        path: PathBuf::from("/h/bin/mr"), target: PathBuf::from("/h/bin/mortar"),
    }];
    assert_eq!(apply_failure("disk failed", &items, "mortar"), Report {
        verb: "install".into(), exit: ExitCode::Failure, outcome: "failed".into(),
        lines: vec!["disk failed".into(),
            "recorded, so mortar uninstall can remove it: /h/bin/mr".into()],
        owner: None, next: "once the failure above is fixed, repeat the same command to finish the install, or run mortar uninstall to remove what was placed".into(), fields: vec![],
    });
}

#[test]
fn recorded_items_use_mortar_for_recovery_when_selected() {
    let items = [
        RecordItem::File { path: PathBuf::from("/h/bin/mortar"), hash: "hash".into() },
        RecordItem::Dir { path: PathBuf::from("/h/bin") },
    ];
    assert_eq!(apply_failure("disk failed", &items, "mortar"), Report {
        verb: "install".into(), exit: ExitCode::Failure, outcome: "failed".into(),
        lines: vec!["disk failed".into(),
            "recorded, so mortar uninstall can remove it: /h/bin/mortar".into(),
            "recorded, so mortar uninstall can remove it: /h/bin".into()],
        owner: None, next: "once the failure above is fixed, repeat the same command to finish the install, or run mortar uninstall to remove what was placed".into(), fields: vec![],
    });
}
