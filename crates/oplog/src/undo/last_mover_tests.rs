//! last_mover: the latest done record after the given index whose move of the ref changes it.

use gitio::Oid;

use super::last_mover;
use crate::store::{OpPhase, OpRecord, RefMove};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn rec(op_id: &str, phase: OpPhase, refname: &str, old: &str, new: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "land".to_string(),
        actor: format!("actor-{op_id}"),
        reason: "test".to_string(),
        at: 1763030400,
        phase,
        moves: vec![RefMove {
            refname: refname.to_string(),
            old: Some(Oid::parse(old).unwrap()),
            new: Oid::parse(new).unwrap(),
        }],
        result: None,
    }
}

fn id(found: Option<&OpRecord>) -> Option<&str> {
    found.map(|r| r.op_id.as_str())
}

#[test]
fn latest_of_two() {
    let records = vec![
        rec("op-1", OpPhase::Done, "refs/heads/main", A, B),
        rec("op-2", OpPhase::Done, "refs/heads/main", B, A),
        rec("op-3", OpPhase::Done, "refs/heads/main", A, B),
    ];
    assert_eq!(id(last_mover(&records, 0, "refs/heads/main")), Some("op-3"));
}

#[test]
fn ignores_records_at_or_before_after() {
    let records = vec![
        rec("op-1", OpPhase::Done, "refs/heads/main", A, B),
        rec("op-2", OpPhase::Done, "refs/heads/main", B, A),
    ];
    assert_eq!(id(last_mover(&records, 1, "refs/heads/main")), None);
}

#[test]
fn ignores_records_not_done() {
    let records = vec![
        rec("op-1", OpPhase::Done, "refs/heads/main", A, B),
        rec("op-2", OpPhase::Done, "refs/heads/main", B, A),
        rec("op-3", OpPhase::Failed, "refs/heads/main", A, B),
        rec("op-4", OpPhase::Intent, "refs/heads/main", A, B),
    ];
    assert_eq!(id(last_mover(&records, 0, "refs/heads/main")), Some("op-2"));
}

#[test]
fn ignores_a_move_that_changes_nothing() {
    let records = vec![
        rec("op-1", OpPhase::Done, "refs/heads/main", A, B),
        rec("op-2", OpPhase::Done, "refs/heads/main", B, A),
        rec("op-3", OpPhase::Done, "refs/heads/main", A, A),
    ];
    assert_eq!(id(last_mover(&records, 0, "refs/heads/main")), Some("op-2"));
}

#[test]
fn no_later_mover_is_none() {
    let records = vec![
        rec("op-1", OpPhase::Done, "refs/heads/main", A, B),
        rec("op-2", OpPhase::Done, "refs/heads/other", B, A),
    ];
    assert_eq!(id(last_mover(&records, 0, "refs/heads/main")), None);
}
