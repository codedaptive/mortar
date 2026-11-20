//! filter_history: by ref (any move), key or actor, in log order.

use gitio::Oid;

use super::{filter_history, HistoryFilter};
use crate::store::{OpPhase, OpRecord, RefMove};

fn mv(refname: &str) -> RefMove {
    RefMove {
        refname: refname.to_string(),
        old: None,
        new: Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(),
    }
}

fn rec(op_id: &str, key: &str, actor: &str, refs: &[&str]) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: key.to_string(),
        kind: "land".to_string(),
        actor: actor.to_string(),
        reason: "squash lands".to_string(),
        at: 1763030400,
        phase: OpPhase::Intent,
        moves: refs.iter().map(|r| mv(r)).collect(),
        result: None,
    }
}

fn log() -> Vec<OpRecord> {
    vec![
        rec("op-1", "land:zone-a", "agent-a", &["refs/heads/landing"]),
        rec("op-2", "land:zone-b", "agent-b", &["refs/heads/main", "refs/heads/landing"]),
        rec("op-3", "land:zone-a", "agent-b", &["refs/heads/main"]),
        rec("op-4", "lock:land", "agent-a", &[]),
    ]
}

fn ids(found: Vec<&OpRecord>) -> Vec<&str> {
    found.into_iter().map(|r| r.op_id.as_str()).collect()
}

#[test]
fn ref_keeps_records_with_any_move_of_that_ref() {
    let log = log();
    let found = filter_history(&log, &HistoryFilter::Ref("refs/heads/landing".to_string()));
    assert_eq!(ids(found), vec!["op-1", "op-2"]);
}

#[test]
fn key_keeps_records_with_that_key() {
    let log = log();
    let found = filter_history(&log, &HistoryFilter::Key("land:zone-a".to_string()));
    assert_eq!(ids(found), vec!["op-1", "op-3"]);
}

#[test]
fn actor_keeps_records_with_that_actor() {
    let log = log();
    let found = filter_history(&log, &HistoryFilter::Actor("agent-a".to_string()));
    assert_eq!(ids(found), vec!["op-1", "op-4"]);
}

#[test]
fn no_match_is_empty() {
    let log = log();
    let found = filter_history(&log, &HistoryFilter::Actor("agent-c".to_string()));
    assert_eq!(ids(found), Vec::<&str>::new());
}

#[test]
fn ref_is_matched_whole_not_by_prefix() {
    let log = log();
    let found = filter_history(&log, &HistoryFilter::Ref("refs/heads/land".to_string()));
    assert_eq!(ids(found), Vec::<&str>::new());
}
