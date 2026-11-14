//! plan_restore: a move to an older value, a delete, a create; no move when already there; a repeated name
//! moved once; UnknownOp with nothing current; ValueUnknown for one ref.

use gitio::Oid;

use super::plan_restore;
use crate::store::{OpPhase, OplogError, OpRecord, RefMove};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const ZERO: &str = "0000000000000000000000000000000000000000";
const MAIN: &str = "refs/heads/main";
const OTHER: &str = "refs/heads/other";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn mv(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove {
        refname: refname.to_string(),
        old: old.map(oid),
        new: oid(new),
    }
}

fn rec(op_id: &str, moves: Vec<RefMove>) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "test".to_string(),
        at: 1763030400,
        phase: OpPhase::Done,
        moves,
        result: None,
    }
}

fn now(refname: &str, value: Option<&str>) -> (String, Option<Oid>) {
    (refname.to_string(), value.map(oid))
}

#[test]
fn moves_to_an_older_value() {
    let records = vec![rec("op-1", vec![mv(MAIN, Some(A), B)]), rec("op-2", vec![mv(MAIN, Some(B), C)])];
    let moves = plan_restore(&records, "op-1", &[now(MAIN, Some(C))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(C), B)]);
}

#[test]
fn a_ref_absent_then_is_deleted() {
    let records = vec![rec("op-1", vec![mv(OTHER, Some(A), B)]), rec("op-2", vec![mv(MAIN, None, C)])];
    let moves = plan_restore(&records, "op-1", &[now(MAIN, Some(C))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(C), ZERO)]);
}

#[test]
fn a_ref_absent_now_is_created() {
    let records = vec![rec("op-1", vec![mv(MAIN, Some(A), B)]), rec("op-2", vec![mv(MAIN, Some(B), ZERO)])];
    let moves = plan_restore(&records, "op-1", &[now(MAIN, None)]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, None, B)]);
}

#[test]
fn a_ref_already_at_its_value_gets_no_move() {
    let records = vec![rec("op-1", vec![mv(MAIN, Some(A), B), mv(OTHER, Some(A), C)])];
    let moves = plan_restore(&records, "op-1", &[now(MAIN, Some(B)), now(OTHER, Some(A))]).unwrap();
    assert_eq!(moves, vec![mv(OTHER, Some(A), C)]);
}

#[test]
fn a_repeated_name_is_moved_once() {
    let records = vec![rec("op-1", vec![mv(MAIN, Some(A), B)])];
    let moves = plan_restore(&records, "op-1", &[now(MAIN, Some(C)), now(MAIN, Some(A))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(C), B)]);
}

#[test]
fn unknown_op_even_with_nothing_current() {
    let records = vec![rec("op-1", vec![mv(MAIN, Some(A), B)])];
    match plan_restore(&records, "op-9", &[]) {
        Err(OplogError::UnknownOp { op_id }) => assert_eq!(op_id, "op-9"),
        other => panic!("expected UnknownOp, got {other:?}"),
    }
}

#[test]
fn value_unknown_for_one_ref() {
    let records = vec![rec("op-1", vec![mv(MAIN, Some(A), B)])];
    match plan_restore(&records, "op-1", &[now(MAIN, Some(C)), now(OTHER, Some(A))]) {
        Err(OplogError::ValueUnknown { op_id, refname }) => {
            assert_eq!(op_id, "op-1");
            assert_eq!(refname, OTHER);
        }
        other => panic!("expected ValueUnknown, got {other:?}"),
    }
}

#[test]
fn restores_to_what_the_ops_closing_record_left() {
    let intent = |op_id: &str, m: RefMove| OpRecord { phase: OpPhase::Intent, ..rec(op_id, vec![m]) };
    let records = vec![
        intent("op-1", mv(MAIN, Some(A), B)),
        rec("op-1", vec![mv(MAIN, Some(A), B)]),
        intent("op-2", mv(MAIN, Some(B), C)),
        rec("op-2", vec![mv(MAIN, Some(B), C)]),
    ];
    let moves = plan_restore(&records, "op-1", &[now(MAIN, Some(C))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(C), B)]);
}
