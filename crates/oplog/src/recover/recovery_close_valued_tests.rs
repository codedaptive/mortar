//! recovery_close_valued: done and failed close the intent with its values kept in order; mixed is the
//! same refusal recovery_close gives.

use gitio::Oid;

use super::{recovery_close_valued, RecoveryDecision};
use crate::store::{OpPhase, OpRecord, OpResult, OplogError, RefMove, ValuedRecord};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn landing_move() -> RefMove {
    RefMove {
        refname: "refs/heads/landing".to_string(),
        old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    }
}

fn record(phase: OpPhase, at: u64, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land:zone-a".to_string(),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "squash lands".to_string(),
        at,
        phase,
        moves: vec![landing_move()],
        result,
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

fn intent(values: &[(&str, &str)]) -> ValuedRecord {
    ValuedRecord { record: record(OpPhase::Intent, 1763030400, None), values: pairs(values) }
}

#[test]
fn done_closes_as_done_moved_keeping_the_values_in_order() {
    let values = [("folder_tree", "3f2a"), ("actor", "simms"), ("folder_tree", "")];
    let r = recovery_close_valued(&intent(&values), RecoveryDecision::Done, 1763030999).unwrap();
    assert_eq!(
        r,
        ValuedRecord {
            record: record(OpPhase::Done, 1763030999, Some(OpResult::Moved)),
            values: pairs(&[("folder_tree", "3f2a"), ("actor", "simms"), ("folder_tree", "")]),
        }
    );
}

#[test]
fn failed_closes_as_failed_not_moved_keeping_the_values_in_order() {
    let refs = vec![(landing_move(), Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")))];
    let values = [("b", "2"), ("a", "1")];
    let r = recovery_close_valued(&intent(&values), RecoveryDecision::Failed(refs), 1763030999).unwrap();
    assert_eq!(
        r,
        ValuedRecord {
            record: record(
                OpPhase::Failed,
                1763030999,
                Some(OpResult::NotMoved {
                    refs: vec![(
                        "refs/heads/landing".to_string(),
                        Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"))
                    )]
                })
            ),
            values: pairs(&[("b", "2"), ("a", "1")]),
        }
    );
}

#[test]
fn an_intent_without_values_closes_without_values() {
    let r = recovery_close_valued(&intent(&[]), RecoveryDecision::Done, 1763030999).unwrap();
    assert_eq!(
        r,
        ValuedRecord {
            record: record(OpPhase::Done, 1763030999, Some(OpResult::Moved)),
            values: vec![],
        }
    );
}

#[test]
fn mixed_is_the_mixed_refusal_naming_op_key_and_refs() {
    let refs = vec![(landing_move(), Some(oid("cccccccccccccccccccccccccccccccccccccccc")))];
    let err = recovery_close_valued(&intent(&[("a", "1")]), RecoveryDecision::Mixed(refs), 1763030999)
        .unwrap_err();
    match err {
        OplogError::Mixed { op_id, key, refs } => {
            assert_eq!(op_id, "op-7");
            assert_eq!(key, "land:zone-a");
            assert_eq!(
                refs,
                vec![(landing_move(), Some(oid("cccccccccccccccccccccccccccccccccccccccc")))]
            );
        }
        other => panic!("expected Mixed, got {other:?}"),
    }
}
