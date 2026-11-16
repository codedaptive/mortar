//! dropped_report: dropped attempts are "dropped"; none left is "already-done"; re-made orphans are named first.

use std::path::PathBuf;

use super::dropped_report;
use crate::report::{ExitCode, OutValue, Report};

fn report(outcome: &str, lines: &[&str], dropped: &[&str], remade: &[&str]) -> Report {
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: None,
        next: "go on with the request in its own folder".to_string(),
        fields: vec![
            ("dropped".to_string(), OutValue::Raw(dropped.iter().map(ToString::to_string).collect())),
            ("remade".to_string(), OutValue::Texts(remade.iter().map(ToString::to_string).collect())),
        ],
    }
}

#[test]
fn two_attempts_dropped() {
    assert_eq!(
        dropped_report(&[1, 2], &[]),
        report(
            "dropped",
            &["dropped attempts 1, 2: their folders and try branches are gone, their trysnap refs kept"],
            &["1", "2"],
            &[]
        )
    );
}

#[test]
fn none_left_is_already_done() {
    assert_eq!(dropped_report(&[], &[]), report("already-done", &["no attempt was left to drop"], &[], &[]));
}

#[test]
fn re_made_orphans_are_named_first() {
    assert_eq!(
        dropped_report(&[1], &[PathBuf::from("/w/tries/try-1")]),
        report(
            "dropped",
            &[
                "re-made the orphan attempt folder /w/tries/try-1",
                "dropped attempts 1: their folders and try branches are gone, their trysnap refs kept",
            ],
            &["1"],
            &["/w/tries/try-1"]
        )
    );
}
