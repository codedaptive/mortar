//! find_pending_move: only a done join-rebase that is the request ref's last mover, with the branch at its new
//! tip, is a pending folder move.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove};

use super::{find_pending_move, PendingMove};

const REQUEST: &str = "refs/heads/req/core/fix-1";
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const LATER: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(op_id: &str, kind: &str, phase: OpPhase, refname: &str, old: Option<&str>, new: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("{kind}:{op_id}"),
        kind: kind.to_string(),
        actor: "agent-a".to_string(),
        reason: "test".to_string(),
        at: 1_700_000_000,
        phase,
        moves: vec![RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }],
        result: None,
    }
}

fn rebase() -> OpRecord {
    record("op-rebase", "join-rebase", OpPhase::Done, REQUEST, Some(OLD), NEW)
}

fn pending() -> PendingMove {
    PendingMove { request_ref: REQUEST.to_string(), old: oid(OLD), new: oid(NEW), op_id: "op-rebase".to_string() }
}

#[test]
fn an_empty_log_has_none() {
    assert_eq!(find_pending_move(&[], REQUEST, Some(&oid(NEW))), None);
}

#[test]
fn a_save_as_last_mover_is_none() {
    let records = [record("op-save", "save", OpPhase::Done, REQUEST, Some(OLD), NEW)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), None);
}

#[test]
fn a_done_join_rebase_with_the_branch_at_its_new_tip_is_the_move() {
    assert_eq!(find_pending_move(&[rebase()], REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_branch_elsewhere_is_none() {
    assert_eq!(find_pending_move(&[rebase()], REQUEST, Some(&oid(LATER))), None);
    assert_eq!(find_pending_move(&[rebase()], REQUEST, Some(&oid(OLD))), None);
}

#[test]
fn an_absent_branch_is_none() {
    assert_eq!(find_pending_move(&[rebase()], REQUEST, None), None);
}

#[test]
fn a_later_save_of_the_same_ref_ends_it() {
    let records = [rebase(), record("op-save", "save", OpPhase::Done, REQUEST, Some(NEW), LATER)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(LATER))), None);
}

#[test]
fn later_failed_and_intent_records_are_ignored() {
    let records = [
        rebase(),
        record("op-failed", "save", OpPhase::Failed, REQUEST, Some(NEW), LATER),
        record("op-intent", "save", OpPhase::Intent, REQUEST, Some(NEW), LATER),
    ];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_later_record_of_another_ref_is_ignored() {
    let records = [rebase(), record("op-zone", "join-ff", OpPhase::Done, "refs/heads/zone/core", Some(NEW), LATER)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_later_move_that_changes_nothing_is_ignored() {
    let records = [rebase(), record("op-same", "save", OpPhase::Done, REQUEST, Some(NEW), NEW)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_join_rebase_without_an_old_tip_is_none() {
    let records = [record("op-rebase", "join-rebase", OpPhase::Done, REQUEST, None, NEW)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), None);
}
