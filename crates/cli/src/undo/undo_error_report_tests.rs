//! undo_error_report: every OplogError arm in the cli's own words, never oplog's or git's Display (S-128 (4)). A ref
//! moved since is refused naming the op, the ref and the op and actor that moved it, or that it moved outside
//! mortar; a protected ref is refused; an open op with its refs half moved is refused, its next in words naming the
//! op; every other arm is a failure. The snapshot is taken before oplog's undo, so a refusal says it is kept.

use std::io;
use std::path::PathBuf;

use gitio::{GitError, Oid};
use oplog::{OplogError, RefMove};

use super::undo_error_report;
use crate::report::{ExitCode, Report};

const REQ: &str = "refs/heads/req/orc/WO-cli-4";
const KEPT: &str = "nothing was undone; the snapshot taken first is kept";

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn refused(lines: &[&str], owner: Option<&str>, next: &str) -> Report {
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: owner.map(str::to_string),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(line: &str) -> Report {
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string(), "nothing half-done is left".to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

fn moved_since(moved_by: Option<(&str, &str)>) -> OplogError {
    OplogError::MovedSince {
        op_id: "op-7".to_string(),
        refname: REQ.to_string(),
        left: Some(oid('c')),
        actual: Some(oid('d')),
        moved_by: moved_by.map(|(op, actor)| (op.to_string(), actor.to_string())),
    }
}

fn malformed_at_line() -> OplogError {
    OplogError::Malformed { line: Some(3), reason: "reason text 3317".to_string() }
}

fn malformed_data() -> OplogError {
    OplogError::Malformed { line: None, reason: "reason text 3317".to_string() }
}

fn io_error() -> OplogError {
    OplogError::Io { path: PathBuf::from("/r/.git/mortar/oplog.jsonl"), source: io::Error::other("disk text 4431") }
}

fn git_error() -> OplogError {
    OplogError::Git(GitError::refusal(vec!["update-ref".to_string()], "fatal: gitio text 5521"))
}

fn mixed() -> OplogError {
    OplogError::Mixed {
        op_id: "op-5".to_string(),
        key: "undo:op-4".to_string(),
        refs: vec![(RefMove { refname: REQ.to_string(), old: Some(oid('b')), new: oid('c') }, Some(oid('d')))],
    }
}

fn bad_move() -> OplogError {
    OplogError::BadMove { refname: "refs/heads/bad..name".to_string(), reason: "reason text 6623".to_string() }
}

fn protected() -> OplogError {
    OplogError::Protected { refname: REQ.to_string(), current: Some(oid('d')), new: oid('b'), by: "kong".to_string() }
}

fn nothing_to_undo() -> OplogError {
    OplogError::NothingToUndo { refs: vec![REQ.to_string(), "refs/mortar/snap/orc/WO-cli-4".to_string()] }
}

fn unknown_op() -> OplogError {
    OplogError::UnknownOp { op_id: "op-3".to_string() }
}

fn value_unknown() -> OplogError {
    OplogError::ValueUnknown { op_id: "op-3".to_string(), refname: REQ.to_string() }
}

fn not_open() -> OplogError {
    OplogError::NotOpen { op_id: "op-3".to_string() }
}

/// One error per arm. The match has no wildcard, so a new OplogError arm fails to compile here until it is added.
fn every_arm() -> Vec<OplogError> {
    let errors = vec![
        malformed_at_line(),
        io_error(),
        git_error(),
        mixed(),
        bad_move(),
        protected(),
        nothing_to_undo(),
        moved_since(Some(("op-9", "orc/other"))),
        unknown_op(),
        value_unknown(),
        not_open(),
    ];
    for error in &errors {
        match error {
            OplogError::Malformed { .. }
            | OplogError::Io { .. }
            | OplogError::Git(_)
            | OplogError::Mixed { .. }
            | OplogError::BadMove { .. }
            | OplogError::Protected { .. }
            | OplogError::NothingToUndo { .. }
            | OplogError::MovedSince { .. }
            | OplogError::UnknownOp { .. }
            | OplogError::ValueUnknown { .. }
            | OplogError::NotOpen { .. } => {}
        }
    }
    errors
}

#[test]
fn a_ref_moved_since_by_a_logged_op_is_refused_naming_that_op_and_its_actor() {
    assert_eq!(
        undo_error_report(&moved_since(Some(("op-9", "orc/other")))),
        refused(
            &["op op-7 cannot be undone: refs/heads/req/orc/WO-cli-4 moved since, by op op-9 of orc/other", KEPT],
            Some("orc/other"),
            "ask the zone's coordinator about refs/heads/req/orc/WO-cli-4",
        )
    );
}

#[test]
fn a_ref_moved_since_outside_mortar_is_refused_saying_so() {
    assert_eq!(
        undo_error_report(&moved_since(None)),
        refused(
            &["op op-7 cannot be undone: refs/heads/req/orc/WO-cli-4 moved since, outside mortar", KEPT],
            None,
            "ask the zone's coordinator about refs/heads/req/orc/WO-cli-4",
        )
    );
}

#[test]
fn a_protected_ref_is_refused_to_its_protector() {
    assert_eq!(
        undo_error_report(&protected()),
        refused(
            &["refs/heads/req/orc/WO-cli-4 is protected by kong, and the undo is not a fast-forward of it", KEPT],
            Some("kong"),
            "ask kong about refs/heads/req/orc/WO-cli-4",
        )
    );
}

#[test]
fn a_malformed_line_is_a_failure_naming_the_line() {
    assert_eq!(
        undo_error_report(&malformed_at_line()),
        failed("line 3 of the op log is not a record mortar wrote")
    );
}

#[test]
fn malformed_data_is_a_failure() {
    assert_eq!(undo_error_report(&malformed_data()), failed("the op log holds data mortar did not write"));
}

#[test]
fn an_io_error_is_a_failure_naming_the_path() {
    assert_eq!(
        undo_error_report(&io_error()),
        failed("the op log at /r/.git/mortar/oplog.jsonl could not be read or written")
    );
}

#[test]
fn a_git_error_is_a_failure() {
    assert_eq!(undo_error_report(&git_error()), failed("git failed while undoing"));
}

/// A repeat meets the same half-moved refs, so the next names the op and its first ref in words (S-112); an agent
/// runs only the six verbs, so no harness command is named.
#[test]
fn an_open_half_moved_op_is_refused_naming_it_and_its_first_ref_for_the_coordinator() {
    assert_eq!(
        undo_error_report(&mixed()),
        refused(
            &["op op-5 is open with its refs neither all moved nor all unmoved", KEPT],
            None,
            "ask the zone's coordinator to settle op op-5 on refs/heads/req/orc/WO-cli-4",
        )
    );
}

#[test]
fn a_bad_move_is_a_failure_naming_the_ref() {
    assert_eq!(
        undo_error_report(&bad_move()),
        failed("the undo would move refs/heads/bad..name, which is not a ref name git accepts")
    );
}

#[test]
fn nothing_to_undo_from_oplog_is_a_failure_naming_the_refs() {
    assert_eq!(
        undo_error_report(&nothing_to_undo()),
        failed("the op log holds no done op that moved refs/heads/req/orc/WO-cli-4, refs/mortar/snap/orc/WO-cli-4")
    );
}

#[test]
fn an_unknown_op_is_a_failure_naming_it() {
    assert_eq!(undo_error_report(&unknown_op()), failed("op op-3 is not in the op log"));
}

#[test]
fn an_unknown_value_is_a_failure_naming_the_op_and_ref() {
    assert_eq!(
        undo_error_report(&value_unknown()),
        failed("the value of refs/heads/req/orc/WO-cli-4 as of op op-3 is unknown")
    );
}

#[test]
fn a_close_of_an_op_not_open_is_a_failure_naming_it() {
    assert_eq!(undo_error_report(&not_open()), failed("op op-3 has no open intent in the op log"));
}

#[test]
fn no_arm_prints_an_inner_errors_text_or_oplogs_display() {
    for error in every_arm().iter().chain([malformed_data(), moved_since(None)].iter()) {
        let printed = format!("{:?}", undo_error_report(error));
        for inner in [
            "3317",
            "4431",
            "5521",
            "6623",
            "mortar did not write it, so check what did",
            "check that path's folder and its permissions",
            "close the op by hand",
            "fix the op's moves and run again",
            "move it only to a commit that has",
            "check the ref names with mortar log",
            "check the ref names with mr log",
            "undo op op-9 first",
            "find the op id with mortar log",
            "find the op id with mr log",
            "name only refs mortar has moved",
            "find the intent line",
        ] {
            assert!(!printed.contains(inner), "{inner:?} in {printed}");
        }
    }
}
