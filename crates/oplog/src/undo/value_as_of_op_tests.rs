//! value_as_of_op: the last done move at or before the op, else the first done move after it (its old);
//! absent as Ok(None); UnknownOp and ValueUnknown; failed and intent records ignored.

use gitio::Oid;

use super::value_as_of_op;
use crate::store::{OpPhase, OplogError, OpRecord, RefMove};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const D: &str = "dddddddddddddddddddddddddddddddddddddddd";
const ZERO: &str = "0000000000000000000000000000000000000000";
const MAIN: &str = "refs/heads/main";
const OTHER: &str = "refs/heads/other";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn rec(op_id: &str, phase: OpPhase, refname: &str, old: Option<&str>, new: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "land".to_string(),
        actor: "agent-a".to_string(),
        reason: "test".to_string(),
        at: 1763030400,
        phase,
        moves: vec![RefMove {
            refname: refname.to_string(),
            old: old.map(oid),
            new: oid(new),
        }],
        result: None,
    }
}

#[test]
fn the_ops_own_done_move() {
    let records = vec![
        rec("op-1", OpPhase::Done, MAIN, Some(A), B),
        rec("op-2", OpPhase::Done, MAIN, Some(B), C),
        rec("op-3", OpPhase::Done, MAIN, Some(C), D),
    ];
    assert_eq!(value_as_of_op(&records, "op-2", MAIN).unwrap(), Some(oid(C)));
}

#[test]
fn an_earlier_ops_move_when_the_op_did_not_touch_the_ref() {
    let records = vec![
        rec("op-1", OpPhase::Done, MAIN, Some(A), B),
        rec("op-2", OpPhase::Done, OTHER, Some(A), C),
        rec("op-3", OpPhase::Done, MAIN, Some(B), D),
    ];
    assert_eq!(value_as_of_op(&records, "op-2", MAIN).unwrap(), Some(oid(B)));
}

#[test]
fn the_first_later_moves_old_when_nothing_before_moved_it() {
    let records = vec![
        rec("op-1", OpPhase::Done, OTHER, Some(A), B),
        rec("op-2", OpPhase::Done, MAIN, Some(C), D),
        rec("op-3", OpPhase::Done, MAIN, Some(D), A),
    ];
    assert_eq!(value_as_of_op(&records, "op-1", MAIN).unwrap(), Some(oid(C)));
}

#[test]
fn a_later_delete_gives_its_old() {
    let records = vec![
        rec("op-1", OpPhase::Done, OTHER, Some(A), B),
        rec("op-2", OpPhase::Done, MAIN, Some(C), ZERO),
    ];
    assert_eq!(value_as_of_op(&records, "op-1", MAIN).unwrap(), Some(oid(C)));
}

#[test]
fn a_later_create_gives_absent() {
    let records = vec![
        rec("op-1", OpPhase::Done, OTHER, Some(A), B),
        rec("op-2", OpPhase::Done, MAIN, None, C),
    ];
    assert_eq!(value_as_of_op(&records, "op-1", MAIN).unwrap(), None);
}

#[test]
fn a_delete_at_or_before_gives_absent() {
    let records = vec![
        rec("op-1", OpPhase::Done, MAIN, Some(A), ZERO),
        rec("op-2", OpPhase::Done, OTHER, Some(A), B),
        rec("op-3", OpPhase::Done, MAIN, None, C),
    ];
    assert_eq!(value_as_of_op(&records, "op-2", MAIN).unwrap(), None);
    assert_eq!(value_as_of_op(&records, "op-1", MAIN).unwrap(), None);
}

#[test]
fn unknown_op() {
    let records = vec![rec("op-1", OpPhase::Done, MAIN, Some(A), B)];
    match value_as_of_op(&records, "op-9", MAIN) {
        Err(OplogError::UnknownOp { op_id }) => assert_eq!(op_id, "op-9"),
        other => panic!("expected UnknownOp, got {other:?}"),
    }
}

#[test]
fn value_unknown_when_no_done_op_moved_the_ref() {
    let records = vec![
        rec("op-1", OpPhase::Done, OTHER, Some(A), B),
        rec("op-2", OpPhase::Failed, MAIN, Some(A), B),
    ];
    match value_as_of_op(&records, "op-1", MAIN) {
        Err(OplogError::ValueUnknown { op_id, refname }) => {
            assert_eq!(op_id, "op-1");
            assert_eq!(refname, MAIN);
        }
        other => panic!("expected ValueUnknown, got {other:?}"),
    }
}

#[test]
fn failed_and_intent_records_are_ignored() {
    let records = vec![
        rec("op-1", OpPhase::Done, MAIN, Some(A), B),
        rec("op-2", OpPhase::Intent, MAIN, Some(B), C),
        rec("op-2", OpPhase::Failed, MAIN, Some(B), C),
        rec("op-3", OpPhase::Intent, MAIN, Some(D), A),
        rec("op-4", OpPhase::Done, MAIN, Some(B), D),
    ];
    assert_eq!(value_as_of_op(&records, "op-2", MAIN).unwrap(), Some(oid(B)));
}

#[test]
fn a_failed_op_names_the_pivot() {
    let records = vec![
        rec("op-1", OpPhase::Failed, OTHER, Some(A), B),
        rec("op-2", OpPhase::Done, MAIN, Some(C), D),
    ];
    assert_eq!(value_as_of_op(&records, "op-1", MAIN).unwrap(), Some(oid(C)));
}

#[test]
fn the_pivot_is_the_ops_closing_record_not_its_intent() {
    let records = vec![
        rec("op-1", OpPhase::Intent, MAIN, Some(A), B),
        rec("op-1", OpPhase::Done, MAIN, Some(A), B),
    ];
    assert_eq!(value_as_of_op(&records, "op-1", MAIN).unwrap(), Some(oid(B)));
}
