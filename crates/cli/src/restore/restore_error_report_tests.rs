//! restore_error_report: the op log's restore refusals are exit 3 with a next step that runs now; anything else is a
//! failure (exit 5). The error's text is passed in, so no oplog code runs here.

use std::path::PathBuf;

use gitio::Oid;
use oplog::OplogError;

use super::restore_error_report;
use crate::report::{ExitCode, Report};

fn refused(line: &str, owner: Option<&str>, next: &str) -> Report {
    Report {
        verb: "restore".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line.to_string()],
        owner: owner.map(ToString::to_string),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn an_unknown_op_points_at_mortar_log() {
    let error = OplogError::UnknownOp { op_id: "op-9".to_string() };
    assert_eq!(restore_error_report(&error, "op op-9 unknown"), refused("op op-9 unknown", None, "mr log"));
}

#[test]
fn an_unknown_value_points_at_the_refs_log() {
    let error = OplogError::ValueUnknown { op_id: "op-9".to_string(), refname: "refs/heads/zone/core".to_string() };
    assert_eq!(
        restore_error_report(&error, "value unknown"),
        refused("value unknown", None, "mr log --ref refs/heads/zone/core")
    );
}

#[test]
fn a_bad_move_names_the_ref_to_fix() {
    let error = OplogError::BadMove { refname: "refs/heads/req/core/a..b".to_string(), reason: "bad".to_string() };
    assert_eq!(
        restore_error_report(&error, "bad move"),
        refused("bad move", None, "name \"refs/heads/req/core/a..b\" by its full, valid ref name, then repeat")
    );
}

#[test]
fn a_protected_ref_is_owned_by_its_protector() {
    let error = OplogError::Protected {
        refname: "refs/heads/main".to_string(),
        current: Some(Oid::parse(&"a".repeat(40)).expect("a valid oid")),
        new: Oid::parse(&"b".repeat(40)).expect("a valid oid"),
        by: "the landing".to_string(),
    };
    assert_eq!(
        restore_error_report(&error, "protected"),
        refused(
            "protected",
            Some("the landing"),
            "restore without refs/heads/main, or ask the landing, which protects it"
        )
    );
}

#[test]
fn any_other_error_is_a_failure() {
    let error =
        OplogError::Io { path: PathBuf::from("/r/.git/mortar/oplog.jsonl"), source: std::io::Error::other("x") };
    let report = restore_error_report(&error, "the disk failed");
    assert_eq!(report.exit, ExitCode::Failure);
    assert_eq!(report.outcome, "failed");
    assert_eq!(report.lines[0], "the disk failed");
    assert_eq!(report.next, "repeat the same command");
}
