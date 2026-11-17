//! kept_report: a keep is "kept"; a keep finished on repeat is "already-done"; re-made orphans are named first.

use std::path::PathBuf;

use gitio::Oid;

use super::kept_report;
use crate::report::{ExitCode, OutValue, Report};

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn report(outcome: &str, lines: Vec<String>, remade: Vec<String>) -> Report {
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next: "go on with the request in its own folder".to_string(),
        fields: vec![
            ("k".to_string(), OutValue::Number(2)),
            ("old".to_string(), OutValue::Text(OLD.to_string())),
            ("new".to_string(), OutValue::Text(NEW.to_string())),
            ("remade".to_string(), OutValue::Texts(remade)),
        ],
    }
}

#[test]
fn a_keep_is_kept() {
    assert_eq!(
        kept_report(2, &oid(OLD), &oid(NEW), &[], false),
        report(
            "kept",
            vec![
                format!("kept attempt 2: the request branch moved from {OLD} to {NEW}"),
                "the request folder holds attempt 2's work; the attempt folders and try branches are gone".to_string(),
            ],
            Vec::new()
        )
    );
}

#[test]
fn a_keep_finished_on_repeat_is_already_done() {
    assert_eq!(
        kept_report(2, &oid(OLD), &oid(NEW), &[], true),
        report(
            "already-done",
            vec![
                format!("attempt 2 was kept before: the request branch had moved from {OLD} to {NEW}"),
                "the request folder holds attempt 2's work; the attempt folders and try branches are gone".to_string(),
            ],
            Vec::new()
        )
    );
}

#[test]
fn re_made_orphans_are_named_first() {
    assert_eq!(
        kept_report(2, &oid(OLD), &oid(NEW), &[PathBuf::from("/w/tries/try-1")], false),
        report(
            "kept",
            vec![
                "re-made the orphan attempt folder /w/tries/try-1".to_string(),
                format!("kept attempt 2: the request branch moved from {OLD} to {NEW}"),
                "the request folder holds attempt 2's work; the attempt folders and try branches are gone".to_string(),
            ],
            vec!["/w/tries/try-1".to_string()]
        )
    );
}
