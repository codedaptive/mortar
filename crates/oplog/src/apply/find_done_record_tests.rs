//! find_done_record: the first done record with the key; intents, failures and other keys do not count.

use super::find_done_record;
use crate::store::{OpPhase, OpRecord, OpResult};

fn rec(op_id: &str, key: &str, phase: OpPhase) -> OpRecord {
    let result = match phase {
        OpPhase::Intent => None,
        OpPhase::Done => Some(OpResult::Moved),
        OpPhase::Failed => Some(OpResult::NotMoved { refs: Vec::new() }),
    };
    OpRecord {
        op_id: op_id.to_string(),
        key: key.to_string(),
        kind: "land".to_string(),
        actor: "agent-a".to_string(),
        reason: "squash lands".to_string(),
        at: 1,
        phase,
        moves: Vec::new(),
        result,
    }
}

#[test]
fn finds_the_done_record_with_the_key() {
    let records = vec![
        rec("op-1", "k", OpPhase::Intent),
        rec("op-1", "k", OpPhase::Done),
    ];
    assert_eq!(
        find_done_record(&records, "k"),
        Some(&rec("op-1", "k", OpPhase::Done))
    );
}

#[test]
fn intent_and_failed_records_with_the_key_are_ignored() {
    let records = vec![
        rec("op-1", "k", OpPhase::Intent),
        rec("op-1", "k", OpPhase::Failed),
        rec("op-2", "k", OpPhase::Intent),
    ];
    assert_eq!(find_done_record(&records, "k"), None);
}

#[test]
fn done_records_with_other_keys_are_ignored() {
    let records = vec![
        rec("op-1", "other", OpPhase::Done),
        rec("op-2", "k-longer", OpPhase::Done),
    ];
    assert_eq!(find_done_record(&records, "k"), None);
}

#[test]
fn the_first_of_two_done_records_is_returned() {
    let records = vec![
        rec("op-1", "k", OpPhase::Done),
        rec("op-2", "k", OpPhase::Done),
    ];
    assert_eq!(find_done_record(&records, "k").map(|r| r.op_id.as_str()), Some("op-1"));
}

#[test]
fn an_empty_log_has_no_done_record() {
    assert_eq!(find_done_record(&[], "k"), None);
}
