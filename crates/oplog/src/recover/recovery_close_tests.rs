//! recovery_close: done and failed close the intent; mixed is the refusal naming the op, key and refs.

use gitio::Oid;

use super::{recovery_close, RecoveryDecision};
use crate::store::{OpPhase, OpRecord, OpResult, OplogError, RefMove};

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

fn intent() -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land:zone-a".to_string(),
        kind: "land".to_string(),
        actor: "agent-a".to_string(),
        reason: "squash lands".to_string(),
        at: 1763030400,
        phase: OpPhase::Intent,
        moves: vec![landing_move()],
        result: None,
    }
}

fn closed(phase: OpPhase, result: OpResult) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land:zone-a".to_string(),
        kind: "land".to_string(),
        actor: "agent-a".to_string(),
        reason: "squash lands".to_string(),
        at: 1763030999,
        phase,
        moves: vec![landing_move()],
        result: Some(result),
    }
}

#[test]
fn done_closes_as_done_moved() {
    let r = recovery_close(&intent(), RecoveryDecision::Done, 1763030999).unwrap();
    assert_eq!(r, closed(OpPhase::Done, OpResult::Moved));
}

#[test]
fn failed_closes_as_failed_not_moved_with_each_ref_and_its_actual_value() {
    let main_move = RefMove {
        refname: "refs/heads/main".to_string(),
        old: Some(oid("cccccccccccccccccccccccccccccccccccccccc")),
        new: oid("dddddddddddddddddddddddddddddddddddddddd"),
    };
    let refs = vec![
        (landing_move(), Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"))),
        (main_move, None),
    ];
    let r = recovery_close(&intent(), RecoveryDecision::Failed(refs), 1763030999).unwrap();
    assert_eq!(
        r,
        closed(
            OpPhase::Failed,
            OpResult::NotMoved {
                refs: vec![
                    (
                        "refs/heads/landing".to_string(),
                        Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"))
                    ),
                    ("refs/heads/main".to_string(), None),
                ]
            }
        )
    );
}

#[test]
fn mixed_is_refused_naming_op_key_and_refs() {
    let refs = vec![(landing_move(), Some(oid("cccccccccccccccccccccccccccccccccccccccc")))];
    let err = recovery_close(&intent(), RecoveryDecision::Mixed(refs), 1763030999).unwrap_err();
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

#[test]
fn mixed_refusal_text_names_the_op_the_ref_and_what_to_do() {
    let refs = vec![(landing_move(), None)];
    let err = recovery_close(&intent(), RecoveryDecision::Mixed(refs), 1763030999).unwrap_err();
    assert_eq!(
        err.to_string(),
        "op op-7 (key land:zone-a) is open and its refs are neither all at their new values nor all at \
         their old ones: refs/heads/landing planned aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa -> \
         bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb, now absent; nothing was changed; find who moved them \
         with mr log --ref <ref>, then close the op by hand with a done or failed record in the op log, \
         since recover refuses every command until it is closed"
    );
}
