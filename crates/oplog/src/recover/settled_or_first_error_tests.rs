//! settled_or_first_error: all Ok gives the records in order; any Err gives the first Err.

use super::settled_or_first_error;
use crate::store::{OpPhase, OpRecord, OpResult, OplogError};

fn closed(op_id: &str, key: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: key.to_string(),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "squash lands".to_string(),
        at: 1763030999,
        phase: OpPhase::Done,
        moves: vec![],
        result: Some(OpResult::Moved),
    }
}

fn malformed(reason: &str) -> OplogError {
    OplogError::Malformed {
        line: None,
        reason: reason.to_string(),
    }
}

#[test]
fn no_results_is_no_records() {
    assert_eq!(settled_or_first_error(vec![]).unwrap(), vec![]);
}

#[test]
fn all_ok_is_every_record_in_order() {
    let got = settled_or_first_error(vec![
        Ok(closed("op-1", "land:zone-a")),
        Ok(closed("op-2", "land:zone-b")),
        Ok(closed("op-3", "land:zone-c")),
    ])
    .unwrap();
    assert_eq!(
        got,
        vec![
            closed("op-1", "land:zone-a"),
            closed("op-2", "land:zone-b"),
            closed("op-3", "land:zone-c"),
        ]
    );
}

#[test]
fn an_error_in_the_middle_is_returned() {
    let got = settled_or_first_error(vec![
        Ok(closed("op-1", "land:zone-a")),
        Err(malformed("ref answer is not an oid")),
        Ok(closed("op-3", "land:zone-c")),
    ]);
    match got {
        Err(OplogError::Malformed { line: None, reason }) => {
            assert_eq!(reason, "ref answer is not an oid")
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn of_two_errors_the_first_is_returned() {
    let got = settled_or_first_error(vec![
        Ok(closed("op-1", "land:zone-a")),
        Err(malformed("first refusal")),
        Err(malformed("second refusal")),
    ]);
    match got {
        Err(OplogError::Malformed { line: None, reason }) => assert_eq!(reason, "first refusal"),
        other => panic!("expected the first Malformed, got {other:?}"),
    }
}
