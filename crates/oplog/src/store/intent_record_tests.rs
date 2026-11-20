//! intent_record: the op's fields and moves, phase Intent, no result.

use gitio::Oid;

use super::{intent_record, Op, OpPhase, OpRecord, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn intent_holds_every_planned_move() {
    let op = Op {
        op_id: "18f3a-2a-0".to_string(),
        key: "join:zone-b".to_string(),
        kind: "join".to_string(),
        actor: "agent-d".to_string(),
        reason: "replay onto integration".to_string(),
        at: 1763030400,
        moves: vec![
            RefMove {
                refname: "refs/heads/a".to_string(),
                old: None,
                new: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            },
            RefMove {
                refname: "refs/heads/b".to_string(),
                old: Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
                new: oid("0000000000000000000000000000000000000000"),
            },
        ],
    };
    assert_eq!(
        intent_record(&op),
        OpRecord {
            op_id: "18f3a-2a-0".to_string(),
            key: "join:zone-b".to_string(),
            kind: "join".to_string(),
            actor: "agent-d".to_string(),
            reason: "replay onto integration".to_string(),
            at: 1763030400,
            phase: OpPhase::Intent,
            moves: vec![
                RefMove {
                    refname: "refs/heads/a".to_string(),
                    old: None,
                    new: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                },
                RefMove {
                    refname: "refs/heads/b".to_string(),
                    old: Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
                    new: oid("0000000000000000000000000000000000000000"),
                },
            ],
            result: None,
        }
    );
}

#[test]
fn op_with_no_moves_gives_intent_with_no_moves() {
    let op = Op {
        op_id: "x".to_string(),
        key: "k".to_string(),
        kind: "lock-takeover".to_string(),
        actor: "1@h".to_string(),
        reason: "r".to_string(),
        at: 0,
        moves: Vec::new(),
    };
    let rec = intent_record(&op);
    assert_eq!(rec.moves, Vec::new());
    assert_eq!(rec.phase, OpPhase::Intent);
    assert_eq!(rec.result, None);
}
