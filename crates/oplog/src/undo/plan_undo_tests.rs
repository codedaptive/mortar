//! plan_undo: an update, a create and a delete reversed; a move that changes nothing left out; a ref moved
//! since refused as MovedSince, naming the latest later mover or none.

use gitio::Oid;

use super::plan_undo;
use crate::store::{OpPhase, OplogError, OpRecord, RefMove};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const ZERO: &str = "0000000000000000000000000000000000000000";
const MAIN: &str = "refs/heads/main";

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

fn rec(op_id: &str, actor: &str, moves: Vec<RefMove>) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "land".to_string(),
        actor: actor.to_string(),
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
fn update_is_reversed() {
    let records = vec![rec("op-1", "agent-a", vec![mv(MAIN, Some(A), B)])];
    let moves = plan_undo(&records, 0, &[now(MAIN, Some(B))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(B), A)]);
}

#[test]
fn create_is_reversed_to_a_delete() {
    let records = vec![rec("op-1", "agent-a", vec![mv(MAIN, None, B)])];
    let moves = plan_undo(&records, 0, &[now(MAIN, Some(B))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(B), ZERO)]);
}

#[test]
fn create_of_a_sha256_ref_is_reversed_to_sixty_four_zeros() {
    let b64 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let zero64 = "0000000000000000000000000000000000000000000000000000000000000000";
    let records = vec![rec("op-1", "agent-a", vec![mv(MAIN, None, b64)])];
    let moves = plan_undo(&records, 0, &[now(MAIN, Some(b64))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(b64), zero64)]);
}

#[test]
fn delete_is_reversed_to_a_create() {
    let records = vec![rec("op-1", "agent-a", vec![mv(MAIN, Some(A), ZERO)])];
    let moves = plan_undo(&records, 0, &[now(MAIN, None)]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, None, A)]);
}

#[test]
fn a_move_that_changes_nothing_is_left_out() {
    let other = "refs/heads/other";
    let records = vec![rec("op-1", "agent-a", vec![mv(other, Some(C), C), mv(MAIN, Some(A), B)])];
    let moves = plan_undo(&records, 0, &[now(other, Some(A)), now(MAIN, Some(B))]).unwrap();
    assert_eq!(moves, vec![mv(MAIN, Some(B), A)]);
}

#[test]
fn moved_since_by_a_logged_op_names_the_latest_mover() {
    let records = vec![
        rec("op-1", "agent-a", vec![mv(MAIN, Some(A), B)]),
        rec("op-2", "agent-b", vec![mv(MAIN, Some(B), C)]),
        rec("op-3", "agent-c", vec![mv(MAIN, Some(C), A)]),
    ];
    match plan_undo(&records, 0, &[now(MAIN, Some(A))]) {
        Err(OplogError::MovedSince { op_id, refname, left, actual, moved_by }) => {
            assert_eq!(op_id, "op-1");
            assert_eq!(refname, MAIN);
            assert_eq!(left, Some(oid(B)));
            assert_eq!(actual, Some(oid(A)));
            assert_eq!(moved_by, Some(("op-3".to_string(), "agent-c".to_string())));
        }
        other => panic!("expected MovedSince, got {other:?}"),
    }
}

#[test]
fn moved_with_no_logged_op_names_no_mover() {
    let records = vec![rec("op-1", "agent-a", vec![mv(MAIN, Some(A), B)])];
    match plan_undo(&records, 0, &[now(MAIN, Some(C))]) {
        Err(OplogError::MovedSince { op_id, refname, left, actual, moved_by }) => {
            assert_eq!(op_id, "op-1");
            assert_eq!(refname, MAIN);
            assert_eq!(left, Some(oid(B)));
            assert_eq!(actual, Some(oid(C)));
            assert_eq!(moved_by, None);
        }
        other => panic!("expected MovedSince, got {other:?}"),
    }
}

#[test]
fn a_created_ref_absent_now_is_moved_since_with_actual_absent() {
    let records = vec![rec("op-1", "agent-a", vec![mv(MAIN, None, B)])];
    match plan_undo(&records, 0, &[]) {
        Err(OplogError::MovedSince { op_id, refname, left, actual, moved_by }) => {
            assert_eq!(op_id, "op-1");
            assert_eq!(refname, MAIN);
            assert_eq!(left, Some(oid(B)));
            assert_eq!(actual, None);
            assert_eq!(moved_by, None);
        }
        other => panic!("expected MovedSince, got {other:?}"),
    }
}

#[test]
fn the_first_moved_ref_in_order_is_refused() {
    let other = "refs/heads/other";
    let records = vec![rec("op-1", "agent-a", vec![mv(MAIN, Some(A), B), mv(other, Some(A), B)])];
    match plan_undo(&records, 0, &[now(other, Some(C)), now(MAIN, Some(C))]) {
        Err(OplogError::MovedSince { refname, .. }) => assert_eq!(refname, MAIN),
        other => panic!("expected MovedSince, got {other:?}"),
    }
}
