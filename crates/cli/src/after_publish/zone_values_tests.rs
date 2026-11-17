//! zone_values: the values a zone branch has held, newest first (S-140): the tip, then for each done record from the
//! newest (the op log is oldest first) each move of the zone's ref, its new then its old; intent and failed records
//! and other refs are skipped, and each value keeps its first place.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove};

use super::zone_values;

const ZONE_REF: &str = "refs/heads/zone/cli";
const TIP: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const A: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const B: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const C: &str = "3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90";
const X: &str = "7f2e4d6c8b0a1f3e5d7c9b2a4f6e8d0c1b3a5f79";
const Y: &str = "b1d3f5a7c9e2b4d6f8a0c3e5b7d9f1a2c4e6b8d0";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn oids(hexes: &[&str]) -> Vec<Oid> {
    hexes.iter().map(|hex| oid(hex)).collect()
}

fn ref_move(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }
}

fn record(op_id: &str, phase: OpPhase, moves: Vec<RefMove>) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "join".to_string(),
        actor: "bilby".to_string(),
        reason: "test".to_string(),
        at: 1_763_251_200,
        phase,
        moves,
        result: None,
    }
}

fn done(op_id: &str, old: Option<&str>, new: &str) -> OpRecord {
    record(op_id, OpPhase::Done, vec![ref_move(ZONE_REF, old, new)])
}

#[test]
fn an_empty_log_gives_the_tip_alone() {
    assert_eq!(zone_values(&[], ZONE_REF, &oid(TIP)), oids(&[TIP]));
}

#[test]
fn the_tip_comes_first_then_each_done_record_from_the_newest_new_before_old() {
    let records = [done("op1", Some(C), A), done("op2", Some(A), B)];
    assert_eq!(zone_values(&records, ZONE_REF, &oid(TIP)), oids(&[TIP, B, A, C]));
}

#[test]
fn an_old_of_none_adds_nothing() {
    let records = [done("op1", None, A), done("op2", Some(A), TIP)];
    assert_eq!(zone_values(&records, ZONE_REF, &oid(TIP)), oids(&[TIP, A]));
}

#[test]
fn intent_and_failed_records_are_skipped() {
    let records = [
        done("op1", Some(C), A),
        record("op2", OpPhase::Intent, vec![ref_move(ZONE_REF, Some(A), X)]),
        record("op2", OpPhase::Failed, vec![ref_move(ZONE_REF, Some(A), X)]),
        record("op3", OpPhase::Intent, vec![ref_move(ZONE_REF, Some(Y), B)]),
    ];
    assert_eq!(zone_values(&records, ZONE_REF, &oid(TIP)), oids(&[TIP, A, C]));
}

#[test]
fn moves_of_other_refs_are_skipped() {
    let records = [
        record(
            "op1",
            OpPhase::Done,
            vec![
                ref_move("refs/heads/zone/clix", Some(X), Y),
                ref_move(ZONE_REF, Some(C), A),
                ref_move("refs/heads/request/cli/r1", None, B),
            ],
        ),
        record("op2", OpPhase::Done, vec![ref_move("refs/heads/zone/land", Some(Y), X)]),
    ];
    assert_eq!(zone_values(&records, ZONE_REF, &oid(TIP)), oids(&[TIP, A, C]));
}

#[test]
fn each_value_keeps_its_first_place() {
    // op3 restored the zone to A; the newest place of A (op4's old) is the one kept.
    let records = [done("op1", None, A), done("op2", Some(A), B), done("op3", Some(B), A), done("op4", Some(A), TIP)];
    assert_eq!(zone_values(&records, ZONE_REF, &oid(TIP)), oids(&[TIP, A, B]));
}

#[test]
fn a_record_moving_the_ref_back_to_the_tip_does_not_repeat_the_tip() {
    let records = [done("op1", Some(TIP), B), done("op2", Some(B), TIP)];
    assert_eq!(zone_values(&records, ZONE_REF, &oid(TIP)), oids(&[TIP, B]));
}
