//! pending_keep_move: only a done keep of attempt k (the request ref moved and try branch k deleted from the new tip
//! in one record) that is the request ref's last mover, with the branch at its new tip, is a pending folder move.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove};

use super::pending_keep_move;

const REQUEST: &str = "refs/heads/req/core/fix-1";
const TRY_1: &str = "refs/heads/try/core/fix-1/1";
const TRY_2: &str = "refs/heads/try/core/fix-1/2";
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const OTHER: &str = "3333333333333333333333333333333333333333";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn mv(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }
}

fn record(kind: &str, phase: OpPhase, moves: Vec<RefMove>) -> OpRecord {
    OpRecord {
        op_id: format!("op-{kind}"),
        key: format!("{kind}:key"),
        kind: kind.to_string(),
        actor: "harness".to_string(),
        reason: "test".to_string(),
        at: 1_700_000_000,
        phase,
        moves,
        result: None,
    }
}

/// keep of attempt 2: the request from OLD to NEW (try 2's tip), try 1 deleted from OTHER, try 2 from NEW.
fn keep_two() -> OpRecord {
    record(
        "keep",
        OpPhase::Done,
        vec![mv(REQUEST, Some(OLD), NEW), mv(TRY_1, Some(OTHER), ZERO), mv(TRY_2, Some(NEW), ZERO)],
    )
}

fn pending() -> Option<(Oid, Oid)> {
    Some((oid(OLD), oid(NEW)))
}

#[test]
fn an_empty_log_has_none() {
    assert_eq!(pending_keep_move(&[], REQUEST, TRY_2, Some(&oid(NEW))), None);
}

#[test]
fn a_done_keep_of_k_with_the_branch_at_its_new_tip_is_the_move() {
    assert_eq!(pending_keep_move(&[keep_two()], REQUEST, TRY_2, Some(&oid(NEW))), pending());
}

#[test]
fn a_keep_that_moves_the_request_to_its_own_tip_still_counts() {
    let same = record("keep", OpPhase::Done, vec![mv(REQUEST, Some(NEW), NEW), mv(TRY_2, Some(NEW), ZERO)]);
    assert_eq!(pending_keep_move(&[same], REQUEST, TRY_2, Some(&oid(NEW))), Some((oid(NEW), oid(NEW))));
}

#[test]
fn a_keep_of_another_attempt_is_none() {
    assert_eq!(pending_keep_move(&[keep_two()], REQUEST, TRY_1, Some(&oid(NEW))), None);
}

#[test]
fn a_record_deleting_try_k_from_another_tip_is_none() {
    let elsewhere = record("keep", OpPhase::Done, vec![mv(REQUEST, Some(OLD), NEW), mv(TRY_2, Some(OTHER), ZERO)]);
    assert_eq!(pending_keep_move(&[elsewhere], REQUEST, TRY_2, Some(&oid(NEW))), None);
}

#[test]
fn a_record_moving_try_k_rather_than_deleting_it_is_none() {
    let moved = record("keep", OpPhase::Done, vec![mv(REQUEST, Some(OLD), NEW), mv(TRY_2, Some(NEW), OTHER)]);
    assert_eq!(pending_keep_move(&[moved], REQUEST, TRY_2, Some(&oid(NEW))), None);
}

#[test]
fn a_branch_elsewhere_or_absent_is_none() {
    assert_eq!(pending_keep_move(&[keep_two()], REQUEST, TRY_2, Some(&oid(OLD))), None);
    assert_eq!(pending_keep_move(&[keep_two()], REQUEST, TRY_2, None), None);
}

#[test]
fn a_request_move_without_an_old_tip_is_none() {
    let made = record("keep", OpPhase::Done, vec![mv(REQUEST, None, NEW), mv(TRY_2, Some(NEW), ZERO)]);
    assert_eq!(pending_keep_move(&[made], REQUEST, TRY_2, Some(&oid(NEW))), None);
}

#[test]
fn a_later_save_of_the_request_ends_it() {
    let save = record("save", OpPhase::Done, vec![mv(REQUEST, Some(NEW), OTHER)]);
    assert_eq!(pending_keep_move(&[keep_two(), save], REQUEST, TRY_2, Some(&oid(OTHER))), None);
}

#[test]
fn a_later_record_that_moves_the_request_to_the_same_tip_ends_it() {
    let same = record("save", OpPhase::Done, vec![mv(REQUEST, Some(NEW), NEW)]);
    assert_eq!(pending_keep_move(&[keep_two(), same], REQUEST, TRY_2, Some(&oid(NEW))), None);
}

#[test]
fn later_intent_and_failed_records_are_skipped() {
    let records = [
        keep_two(),
        record("save", OpPhase::Intent, vec![mv(REQUEST, Some(NEW), OTHER)]),
        record("save", OpPhase::Failed, vec![mv(REQUEST, Some(NEW), OTHER)]),
    ];
    assert_eq!(pending_keep_move(&records, REQUEST, TRY_2, Some(&oid(NEW))), pending());
}

#[test]
fn a_later_record_of_other_refs_only_is_skipped() {
    let zone = record("join-ff", OpPhase::Done, vec![mv("refs/heads/zone/core", Some(OLD), OTHER)]);
    assert_eq!(pending_keep_move(&[keep_two(), zone], REQUEST, TRY_2, Some(&oid(NEW))), pending());
}

#[test]
fn an_intent_of_a_keep_moved_nothing_and_is_none() {
    let intent = record("keep", OpPhase::Intent, keep_two().moves);
    assert_eq!(pending_keep_move(&[intent], REQUEST, TRY_2, Some(&oid(NEW))), None);
}
