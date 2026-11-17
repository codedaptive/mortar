//! last_integration_old: the old value of the last done move that changed the integration ref; intent and failed
//! records, other refs and moves that change nothing are skipped.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove};

use super::last_integration_old;

const INTEGRATION: &str = "refs/heads/integration";
const ONE: &str = "1111111111111111111111111111111111111111";
const TWO: &str = "2222222222222222222222222222222222222222";
const THREE: &str = "3333333333333333333333333333333333333333";
const FOUR: &str = "4444444444444444444444444444444444444444";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn mv(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }
}

fn record(op_id: &str, phase: OpPhase, moves: Vec<RefMove>) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("land:{op_id}"),
        kind: "land".to_string(),
        actor: "harness".to_string(),
        reason: "test".to_string(),
        at: 1_700_000_000,
        phase,
        moves,
        result: None,
    }
}

fn done(op_id: &str, old: Option<&str>, new: &str) -> OpRecord {
    record(op_id, OpPhase::Done, vec![mv(INTEGRATION, old, new)])
}

#[test]
fn an_empty_log_is_none() {
    assert_eq!(last_integration_old(&[], INTEGRATION), None);
}

#[test]
fn a_done_move_of_integration_gives_its_old() {
    assert_eq!(last_integration_old(&[done("op-1", Some(ONE), TWO)], INTEGRATION), Some(oid(ONE)));
}

#[test]
fn the_last_of_several_done_moves_wins() {
    let records = [done("op-1", Some(ONE), TWO), done("op-2", Some(TWO), THREE), done("op-3", Some(THREE), FOUR)];
    assert_eq!(last_integration_old(&records, INTEGRATION), Some(oid(THREE)));
}

#[test]
fn intent_and_failed_records_are_skipped() {
    let records = [
        done("op-1", Some(ONE), TWO),
        record("op-2", OpPhase::Intent, vec![mv(INTEGRATION, Some(TWO), THREE)]),
        record("op-3", OpPhase::Failed, vec![mv(INTEGRATION, Some(TWO), FOUR)]),
    ];
    assert_eq!(last_integration_old(&records, INTEGRATION), Some(oid(ONE)));
}

#[test]
fn only_intent_and_failed_records_is_none() {
    let records = [
        record("op-1", OpPhase::Intent, vec![mv(INTEGRATION, Some(ONE), TWO)]),
        record("op-1", OpPhase::Failed, vec![mv(INTEGRATION, Some(ONE), TWO)]),
    ];
    assert_eq!(last_integration_old(&records, INTEGRATION), None);
}

#[test]
fn moves_of_other_refs_are_skipped() {
    let records = [
        done("op-1", Some(ONE), TWO),
        record("op-2", OpPhase::Done, vec![mv("refs/heads/main", Some(TWO), THREE)]),
        record("op-3", OpPhase::Done, vec![mv("refs/heads/integration-old", Some(THREE), FOUR)]),
    ];
    assert_eq!(last_integration_old(&records, INTEGRATION), Some(oid(ONE)));
}

#[test]
fn only_other_refs_is_none() {
    let records = [record("op-1", OpPhase::Done, vec![mv("refs/heads/main", Some(ONE), TWO)])];
    assert_eq!(last_integration_old(&records, INTEGRATION), None);
}

#[test]
fn the_integration_move_is_found_among_a_records_other_moves() {
    let moves = vec![
        mv("refs/heads/main", Some(ONE), TWO),
        mv(INTEGRATION, Some(THREE), FOUR),
        mv("refs/mortar/squash/core", Some(TWO), ONE),
    ];
    assert_eq!(last_integration_old(&[record("op-1", OpPhase::Done, moves)], INTEGRATION), Some(oid(THREE)));
}

#[test]
fn a_later_move_that_changes_nothing_is_skipped() {
    let records = [done("op-1", Some(ONE), TWO), done("op-2", Some(TWO), TWO)];
    assert_eq!(last_integration_old(&records, INTEGRATION), Some(oid(ONE)));
}

#[test]
fn only_moves_that_change_nothing_is_none() {
    assert_eq!(last_integration_old(&[done("op-1", Some(TWO), TWO)], INTEGRATION), None);
}

#[test]
fn a_last_move_without_an_old_is_none() {
    let records = [done("op-1", Some(ONE), TWO), done("op-2", None, THREE)];
    assert_eq!(last_integration_old(&records, INTEGRATION), None);
}

#[test]
fn a_later_move_with_an_old_wins_over_one_without() {
    let records = [done("op-1", None, ONE), done("op-2", Some(ONE), TWO)];
    assert_eq!(last_integration_old(&records, INTEGRATION), Some(oid(ONE)));
}
