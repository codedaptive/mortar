//! Display for WithdrawError's own refusals, each as literal text saying nothing moved: Published and Landing name
//! the squash commit and the next step, OpFailed and OpRepeated the op. The Squash, Git, Oplog and State arms embed
//! another error's text and have no case here.

use super::WithdrawError;
use gitio::Oid;
use oplog::{ChangeId, OpPhase, OpRecord, OpResult};

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-withdraw:refs/mortar/squash/land:a:op-7".to_string(),
        kind: "land-withdraw".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

#[test]
fn each_refusal_names_the_commit_or_op_and_says_nothing_moved() {
    let change_id = ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap();
    let rows = [
        (
            WithdrawError::Published { zone: "land".to_string(), commit: a(), origin: b() },
            "squash aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa of zone land is already in origin (at \
             bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb), so it was published and cannot be withdrawn; nothing was \
             moved; next: commit new work on the zone, then mr squash land starts a new change",
        ),
        (
            WithdrawError::Landing { zone: "land".to_string(), commit: a(), change_id },
            "squash aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa of zone land is the landing in flight \
             (klmnopqrstuvwxyzklmnopqrstuvwxyz), so it cannot be withdrawn; nothing was moved; next: mr back-out, \
             then mr withdraw land",
        ),
        (
            WithdrawError::OpFailed(record(OpPhase::Failed)),
            "op op-7 (key land-withdraw:refs/mortar/squash/land:a:op-7) failed: the squash ref was not at the \
             commit read, so nothing was moved; run again to read it afresh",
        ),
        (
            WithdrawError::OpRepeated(record(OpPhase::Done)),
            "op key land-withdraw:refs/mortar/squash/land:a:op-7 is already done in the op log, so this attempt \
             moved nothing; run again",
        ),
    ];
    for (error, expected) in rows {
        assert_eq!(error.to_string(), expected, "{error:?}");
    }
}
