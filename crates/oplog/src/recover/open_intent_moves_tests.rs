//! open_intent_moves: an open intent's moves for its op_id; None once a later Done or Failed closes it, or
//! for an op_id with no intent. These run open_intent_moves with the real open_intents, a pure filter over
//! the record slice, so they are unit tests (S-166).

use gitio::Oid;

use super::open_intent_moves;
use crate::store::{OpPhase, OpRecord, OpResult, RefMove};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const D: &str = "dddddddddddddddddddddddddddddddddddddddd";
const E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn mv(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove {
        refname: refname.to_string(),
        old: old.map(oid),
        new: oid(new),
    }
}

fn rec(op_id: &str, phase: OpPhase, moves: Vec<RefMove>) -> OpRecord {
    let result = match phase {
        OpPhase::Intent => None,
        OpPhase::Done => Some(OpResult::Moved),
        OpPhase::Failed => Some(OpResult::NotMoved { refs: Vec::new() }),
    };
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "squash lands".to_string(),
        at: 1763030400,
        phase,
        moves,
        result,
    }
}

fn land_moves() -> Vec<RefMove> {
    vec![
        mv("refs/heads/integration", Some(A), B),
        mv("refs/heads/landing", None, C),
    ]
}

#[test]
fn an_open_intent_gives_its_moves() {
    let log = vec![rec("op-1", OpPhase::Intent, land_moves())];
    assert_eq!(
        open_intent_moves(&log, "op-1"),
        Some(vec![
            RefMove {
                refname: "refs/heads/integration".to_string(),
                old: Some(oid(A)),
                new: oid(B),
            },
            RefMove {
                refname: "refs/heads/landing".to_string(),
                old: None,
                new: oid(C),
            },
        ])
    );
}

#[test]
fn an_intent_closed_by_done_gives_none() {
    let log = vec![
        rec("op-1", OpPhase::Intent, land_moves()),
        rec("op-1", OpPhase::Done, land_moves()),
    ];
    assert_eq!(open_intent_moves(&log, "op-1"), None);
}

#[test]
fn an_intent_closed_by_failed_gives_none() {
    let log = vec![
        rec("op-1", OpPhase::Intent, land_moves()),
        rec("op-1", OpPhase::Failed, land_moves()),
    ];
    assert_eq!(open_intent_moves(&log, "op-1"), None);
}

#[test]
fn an_unknown_op_id_gives_none() {
    let log = vec![rec("op-1", OpPhase::Intent, land_moves())];
    assert_eq!(open_intent_moves(&log, "op-9"), None);
}

#[test]
fn only_the_asked_ops_moves_come_back() {
    let log = vec![
        rec("op-1", OpPhase::Intent, vec![mv("refs/heads/main", Some(A), B)]),
        rec("op-2", OpPhase::Intent, vec![mv("refs/heads/integration", Some(C), D)]),
        rec("op-3", OpPhase::Intent, vec![mv("refs/heads/landing", Some(D), E)]),
        rec("op-3", OpPhase::Done, vec![mv("refs/heads/landing", Some(D), E)]),
    ];
    assert_eq!(
        open_intent_moves(&log, "op-2"),
        Some(vec![RefMove {
            refname: "refs/heads/integration".to_string(),
            old: Some(oid(C)),
            new: oid(D),
        }])
    );
    assert_eq!(open_intent_moves(&log, "op-3"), None);
}

#[test]
fn a_done_before_the_intent_does_not_close_it() {
    let log = vec![
        rec("op-1", OpPhase::Done, vec![mv("refs/heads/main", Some(A), B)]),
        rec("op-1", OpPhase::Intent, vec![mv("refs/heads/main", Some(B), C)]),
    ];
    assert_eq!(
        open_intent_moves(&log, "op-1"),
        Some(vec![RefMove {
            refname: "refs/heads/main".to_string(),
            old: Some(oid(B)),
            new: oid(C),
        }])
    );
}

#[test]
fn an_open_intent_with_no_moves_gives_an_empty_list() {
    let log = vec![rec("op-1", OpPhase::Intent, vec![])];
    assert_eq!(open_intent_moves(&log, "op-1"), Some(vec![]));
}
