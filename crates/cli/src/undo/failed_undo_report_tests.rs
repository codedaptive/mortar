//! failed_undo_report: an undo op that found its ref moved is refused naming the op undone and the ref, moved
//! outside mortar; a failed record without a stale ref is a failure naming the undo op.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove};

use super::failed_undo_report;
use crate::report::{ExitCode, Report};

const REQ: &str = "refs/heads/req/a/r1";

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn failed(result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-12".to_string(),
        key: "undo:op-7".to_string(),
        kind: "undo".to_string(),
        actor: "a/r1".to_string(),
        reason: "undo of save op op-7".to_string(),
        at: 50,
        phase: OpPhase::Failed,
        moves: vec![RefMove { refname: REQ.to_string(), old: Some(oid('c')), new: oid('b') }],
        result,
    }
}

#[test]
fn a_stale_ref_is_refused_naming_the_op_undone_and_the_ref_moved_outside_mortar() {
    let stale = Some(OpResult::Stale { refname: REQ.to_string(), actual: Some(oid('d')) });
    assert_eq!(
        failed_undo_report("op-7", &failed(stale)),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "op op-7 cannot be undone: refs/heads/req/a/r1 moved since, outside mortar".to_string(),
                "nothing was undone; the snapshot taken first is kept".to_string(),
            ],
            owner: None,
            next: "ask the zone's coordinator about refs/heads/req/a/r1".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_failed_record_with_no_stale_ref_is_a_failure_naming_the_undo_op() {
    for result in [None, Some(OpResult::Moved), Some(OpResult::NotMoved { refs: Vec::new() })] {
        let report = failed_undo_report("op-7", &failed(result));
        assert_eq!(report.exit, ExitCode::Failure);
        assert_eq!(report.lines[0], "the undo op op-12 failed and moved nothing");
    }
}
