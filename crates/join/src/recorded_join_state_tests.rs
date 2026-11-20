//! recorded_join_state: a good resolve op gives its state, whatever its move's new value (no ref is read); each cause
//! of NotAResolveOp (another kind, an intent, two moves, no value, a plain join's op with only folder_tree); the first
//! of two join_state values taken; each cause of RecordedStateCorrupt. Ops are built as data; the recorded texts are
//! written out by hand.

use crate::join_state::JoinState;
use crate::recorded_join_state::recorded_join_state;
use crate::JoinRunError;
use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove, ValuedRecord};
use std::path::PathBuf;

fn oid(digit: char) -> Oid {
    Oid::parse(&digit.to_string().repeat(40)).unwrap()
}

const REQUEST: &str = "refs/heads/request/r1";

/// The text join_state_value records for `expected()`.
const RECORDED: &str = concat!(
    "{\"request_ref\":\"refs/heads/request/r1\",",
    "\"request_tip\":\"1111111111111111111111111111111111111111\",",
    "\"onto\":\"2222222222222222222222222222222222222222\",",
    "\"replayed_tip\":\"3333333333333333333333333333333333333333\",",
    "\"commit\":\"4444444444444444444444444444444444444444\",",
    "\"tree\":\"5555555555555555555555555555555555555555\",",
    "\"remaining\":[\"6666666666666666666666666666666666666666\"],",
    "\"files\":[\"a.txt\"],\"all_files\":[\"a.txt\",\"dir/b.txt\"],",
    "\"files_written\":true,\"rebased_to\":null,\"folder_tree\":null}"
);

fn expected() -> JoinState {
    JoinState {
        request_ref: REQUEST.to_string(),
        request_tip: oid('1'),
        onto: oid('2'),
        replayed_tip: oid('3'),
        commit: oid('4'),
        tree: oid('5'),
        remaining: vec![oid('6')],
        files: vec![PathBuf::from("a.txt")],
        all_files: vec![PathBuf::from("a.txt"), PathBuf::from("dir/b.txt")],
        files_written: true,
        rebased_to: None,
        folder_tree: None,
    }
}

fn request_move(old: Option<Oid>) -> RefMove {
    RefMove { refname: REQUEST.to_string(), old, new: oid('8') }
}

/// A resolve's final op: done join-rebase, one move of the request ref from tip 1, folder_tree then join_state.
fn resolve_op() -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: "op-20".to_string(),
            key: "join-rebase:refs/heads/request/r1:op-20".to_string(),
            kind: "join-rebase".to_string(),
            actor: "42@host".to_string(),
            reason: "resolve".to_string(),
            at: 9,
            phase: OpPhase::Done,
            moves: vec![request_move(Some(oid('1')))],
            result: Some(OpResult::Moved),
        },
        values: vec![
            ("folder_tree".to_string(), "9".repeat(40)),
            ("join_state".to_string(), RECORDED.to_string()),
        ],
    }
}

fn with_state(text: &str) -> ValuedRecord {
    ValuedRecord { values: vec![("join_state".to_string(), text.to_string())], ..resolve_op() }
}

fn assert_not_a_resolve_op(op: ValuedRecord) {
    match recorded_join_state(&op) {
        Err(JoinRunError::NotAResolveOp { op_id }) => assert_eq!(op_id, "op-20"),
        other => panic!("expected NotAResolveOp, got {other:?}"),
    }
}

fn corrupt_reason(op: ValuedRecord) -> String {
    match recorded_join_state(&op) {
        Err(JoinRunError::RecordedStateCorrupt { op_id, reason }) => {
            assert_eq!(op_id, "op-20");
            reason
        }
        other => panic!("expected RecordedStateCorrupt, got {other:?}"),
    }
}

#[test]
fn a_resolves_final_op_gives_the_recorded_state() {
    assert_eq!(recorded_join_state(&resolve_op()).unwrap(), expected());
}

#[test]
fn a_good_op_gives_the_recorded_state_whatever_its_moves_new_value() {
    for new in [oid('8'), oid('1'), oid('f')] {
        let mut op = resolve_op();
        op.record.moves[0].new = new;
        assert_eq!(recorded_join_state(&op).unwrap(), expected());
    }
}

#[test]
fn another_kind_is_not_a_resolve_op() {
    let mut op = resolve_op();
    op.record.kind = "save".to_string();
    assert_not_a_resolve_op(op);
}

#[test]
fn an_intent_or_failed_op_is_not_a_resolve_op() {
    for phase in [OpPhase::Intent, OpPhase::Failed] {
        let mut op = resolve_op();
        op.record.phase = phase;
        assert_not_a_resolve_op(op);
    }
}

#[test]
fn two_moves_or_none_is_not_a_resolve_op() {
    let mut two = resolve_op();
    two.record.moves.push(RefMove { refname: "refs/heads/zone/join".to_string(), old: Some(oid('2')), new: oid('7') });
    assert_not_a_resolve_op(two);
    let mut none = resolve_op();
    none.record.moves.clear();
    assert_not_a_resolve_op(none);
}

#[test]
fn no_values_is_not_a_resolve_op() {
    assert_not_a_resolve_op(ValuedRecord { values: vec![], ..resolve_op() });
}

#[test]
fn a_plain_joins_op_with_only_folder_tree_is_not_a_resolve_op() {
    assert_not_a_resolve_op(ValuedRecord { values: vec![("folder_tree".to_string(), "9".repeat(40))], ..resolve_op() });
}

#[test]
fn the_first_of_two_join_state_values_is_taken() {
    let other = RECORDED.replace("\"files_written\":true", "\"files_written\":false");
    let op = ValuedRecord {
        values: vec![("join_state".to_string(), RECORDED.to_string()), ("join_state".to_string(), other)],
        ..resolve_op()
    };
    assert_eq!(recorded_join_state(&op).unwrap(), expected());
}

#[test]
fn a_value_that_does_not_decode_is_corrupt_with_the_decoders_reason() {
    assert_eq!(corrupt_reason(with_state("not a state")), "expected {\"request_ref\": at byte 0");
}

#[test]
fn a_recorded_rebased_to_or_folder_tree_is_corrupt() {
    let ninth = "\"9999999999999999999999999999999999999999\"";
    let rebased = RECORDED.replace("\"rebased_to\":null", &format!("\"rebased_to\":{ninth}"));
    let tree = RECORDED.replace("\"folder_tree\":null", &format!("\"folder_tree\":{ninth}"));
    for text in [rebased, tree] {
        let reason = corrupt_reason(with_state(&text));
        assert!(reason.contains("rebased_to or folder_tree set"), "{reason}");
    }
}

#[test]
fn a_recorded_request_ref_other_than_the_moved_ref_is_corrupt() {
    let reason = corrupt_reason(with_state(&RECORDED.replace("request/r1", "request/r2")));
    assert_eq!(
        reason,
        "the recorded state's request ref refs/heads/request/r2 is not the op's moved ref refs/heads/request/r1"
    );
}

#[test]
fn a_move_not_from_the_recorded_request_tip_is_corrupt() {
    let mut elsewhere = resolve_op();
    elsewhere.record.moves = vec![request_move(Some(oid('7')))];
    assert_eq!(
        corrupt_reason(elsewhere),
        "the op moved refs/heads/request/r1 from 7777777777777777777777777777777777777777, not from the recorded \
         state's request tip 1111111111111111111111111111111111111111"
    );
    let mut created = resolve_op();
    created.record.moves = vec![request_move(None)];
    assert_eq!(
        corrupt_reason(created),
        "the op moved refs/heads/request/r1 from nothing, not from the recorded state's request tip \
         1111111111111111111111111111111111111111"
    );
}
