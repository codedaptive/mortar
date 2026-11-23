//! joined_zone_base: a done join-zone-request, last mover of the request ref with the branch at its request move's new
//! tip, gives its zone move's old tip, or that new tip when the op has no zone move; any other last mover, a branch
//! elsewhere and a request move with no old tip give none.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove, ValuedRecord};

use super::joined_zone_base;

const REQUEST: &str = "refs/heads/req/core/fix-1";
const ZONE: &str = "refs/heads/zone/core";
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const LATER: &str = "3333333333333333333333333333333333333333";
const ZONE_FROM: &str = "6666666666666666666666666666666666666666";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn one(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }
}

fn record(op_id: &str, kind: &str, moves: Vec<RefMove>) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("{kind}:{op_id}"),
            kind: kind.to_string(),
            actor: "agent-a".to_string(),
            reason: "test".to_string(),
            at: 1_700_000_000,
            phase: OpPhase::Done,
            moves,
            result: None,
        },
        values: Vec::new(),
    }
}

/// join_on_base's joined partner request: the zone from ZONE_FROM to NEW, then the request from OLD to NEW.
fn joined() -> ValuedRecord {
    record("op-joined", "join-zone-request", vec![one(ZONE, Some(ZONE_FROM), NEW), one(REQUEST, Some(OLD), NEW)])
}

#[test]
fn joined_zone_base_cases() {
    let save = record("op-save", "save", vec![one(REQUEST, Some(NEW), LATER)]);
    let rebase = record("op-rebase", "join-rebase", vec![one(REQUEST, Some(OLD), NEW)]);
    let later_rebase = record("op-later", "join-rebase", vec![one(REQUEST, Some(NEW), LATER)]);
    let no_zone_move = record("op-joined", "join-zone-request", vec![one(REQUEST, Some(OLD), NEW)]);
    let no_old = record("op-joined", "join-zone-request", vec![one(ZONE, Some(ZONE_FROM), NEW), one(REQUEST, None, NEW)]);
    let cases: [(&str, Vec<ValuedRecord>, &str, Option<&str>); 7] = [
        ("a joined request with the branch at its new tip", vec![joined()], NEW, Some(ZONE_FROM)),
        ("a joined request with no zone move", vec![no_zone_move], NEW, Some(NEW)),
        ("the branch elsewhere", vec![joined()], LATER, None),
        ("a join rebase as last mover", vec![rebase], NEW, None),
        ("a later join rebase of the request", vec![joined(), later_rebase], LATER, None),
        ("a later save", vec![joined(), save], LATER, None),
        ("a request move with no old tip", vec![no_old], NEW, None),
    ];
    for (case, records, branch_at, expected) in cases {
        assert_eq!(joined_zone_base(&records, REQUEST, Some(&oid(branch_at))), expected.map(oid), "{case}");
    }
    assert_eq!(joined_zone_base(&[joined()], REQUEST, None), None, "an absent branch");
}
