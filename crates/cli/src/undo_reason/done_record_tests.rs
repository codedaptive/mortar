//! done_record: the last Done record of an op id in log order; Intent and Failed records and other ids are skipped.

use oplog::{OpPhase, OpRecord, ValuedRecord};

use super::done_record;

fn record(op_id: &str, phase: OpPhase, reason: &str) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("join-rebase:{op_id}"),
            kind: "join-rebase".to_string(),
            actor: "agent-a".to_string(),
            reason: reason.to_string(),
            at: 1_700_000_000,
            phase,
            moves: Vec::new(),
            result: None,
        },
        values: Vec::new(),
    }
}

fn reason_of(found: Option<&ValuedRecord>) -> Option<&str> {
    found.map(|valued| valued.record.reason.as_str())
}

#[test]
fn an_empty_log_has_none() {
    assert_eq!(reason_of(done_record(&[], "op-5")), None);
}

#[test]
fn intent_and_failed_records_of_the_id_are_skipped() {
    let records = [record("op-5", OpPhase::Intent, "intent"), record("op-5", OpPhase::Failed, "failed")];
    assert_eq!(reason_of(done_record(&records, "op-5")), None);
}

#[test]
fn a_done_record_beside_its_intent_and_a_later_failed_one_is_found() {
    let records = [
        record("op-5", OpPhase::Intent, "intent"),
        record("op-5", OpPhase::Done, "done"),
        record("op-5", OpPhase::Failed, "failed"),
    ];
    assert_eq!(reason_of(done_record(&records, "op-5")), Some("done"));
}

#[test]
fn the_last_of_two_done_records_of_the_id_is_found() {
    let records = [record("op-5", OpPhase::Done, "first"), record("op-5", OpPhase::Done, "second")];
    assert_eq!(reason_of(done_record(&records, "op-5")), Some("second"));
}

#[test]
fn done_records_of_other_ids_are_not_found() {
    let records = [
        record("op-4", OpPhase::Done, "four"),
        record("op-5", OpPhase::Done, "five"),
        record("op-50", OpPhase::Done, "fifty"),
    ];
    assert_eq!(reason_of(done_record(&records, "op-5")), Some("five"));
    assert_eq!(reason_of(done_record(&records, "op-6")), None);
}
