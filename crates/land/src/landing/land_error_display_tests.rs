//! Display for LandError's own refusals that carry only op-log records (OpFailed, OpRepeated), each as literal text
//! naming the op and saying nothing moved. The arms that print an Oid (NoZone, NoLandingState, IntegrationElsewhere)
//! are Nagatha's RITs (owed, S-101); the State arm, which runs
//! LandingStateError's Display, is tested in tests/landing_zit (state_display); the Squash, Git, Oplog and Trailers
//! arms embed another error's text and have no case here.

use super::LandError;
use oplog::{OpPhase, OpRecord, OpResult};

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land:refs/heads/integration:a:b:op-7".to_string(),
        kind: "land".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

#[test]
fn op_failed_names_the_op_and_its_key_and_says_nothing_moved() {
    assert_eq!(
        LandError::OpFailed(record(OpPhase::Failed)).to_string(),
        "op op-7 (key land:refs/heads/integration:a:b:op-7) failed: integration was not at its expected value, so \
         nothing was moved; run again to read the refs afresh"
    );
}

#[test]
fn op_repeated_names_the_key_and_says_nothing_moved() {
    assert_eq!(
        LandError::OpRepeated(record(OpPhase::Done)).to_string(),
        "op key land:refs/heads/integration:a:b:op-7 is already done in the op log, so this attempt moved nothing; \
         run again"
    );
}

