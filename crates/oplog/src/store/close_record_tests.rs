//! close_record: the intent's identity and moves, with the given phase, result and time.

use gitio::Oid;

use super::{close_record, OpPhase, OpRecord, OpResult, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn intent() -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "land:zone-a".to_string(),
        kind: "land".to_string(),
        actor: "agent-a".to_string(),
        reason: "squash lands".to_string(),
        at: 1763030400,
        phase: OpPhase::Intent,
        moves: vec![RefMove {
            refname: "refs/heads/landing".to_string(),
            old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
            new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        }],
        result: None,
    }
}

#[test]
fn done_keeps_identity_and_moves() {
    assert_eq!(
        close_record(&intent(), OpPhase::Done, OpResult::Moved, 1763030405),
        OpRecord {
            op_id: "op-1".to_string(),
            key: "land:zone-a".to_string(),
            kind: "land".to_string(),
            actor: "agent-a".to_string(),
            reason: "squash lands".to_string(),
            at: 1763030405,
            phase: OpPhase::Done,
            moves: vec![RefMove {
                refname: "refs/heads/landing".to_string(),
                old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
                new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            }],
            result: Some(OpResult::Moved),
        }
    );
}

#[test]
fn failed_carries_the_stale_result() {
    let stale = OpResult::Stale {
        refname: "refs/heads/landing".to_string(),
        actual: Some(oid("cccccccccccccccccccccccccccccccccccccccc")),
    };
    let closed = close_record(&intent(), OpPhase::Failed, stale, 7);
    assert_eq!(closed.phase, OpPhase::Failed);
    assert_eq!(closed.at, 7);
    assert_eq!(
        closed.result,
        Some(OpResult::Stale {
            refname: "refs/heads/landing".to_string(),
            actual: Some(oid("cccccccccccccccccccccccccccccccccccccccc")),
        })
    );
    assert_eq!(closed.op_id, "op-1");
    assert_eq!(closed.moves.len(), 1);
}
