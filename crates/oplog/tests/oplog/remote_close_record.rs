//! Joins close_intent -> read_marked_log -> close_intent_record (open_intents_in, close_record) on a log
//! seeded with append_marked_record: Done and Failed close the open intent with the op_id, keeping its
//! values and mark, and append that record; an op_id with no open intent is NotOpen and appends nothing.
//! The closing record's `at` is the clock (unix_now), so it is checked against reads before and after.

use crate::support::Fixture;
use gitio::Oid;
use oplog::{
    append_marked_record, close_intent, oplog_path, read_marked_log, unix_now, IntentClose, MarkedRecord, OpPhase,
    OpRecord, OpResult, OplogError, RefMove,
};
use std::fs;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(op_id: &str, phase: OpPhase, at: u64, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("publish:{op_id}"),
        kind: "publish".to_string(),
        actor: "simms".to_string(),
        reason: "squash publishes".to_string(),
        at,
        phase,
        moves: vec![RefMove {
            refname: "refs/heads/main".to_string(),
            old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
            new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        }],
        result,
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

fn intent(op_id: &str, values: &[(&str, &str)], remote: Option<&str>) -> MarkedRecord {
    MarkedRecord {
        record: record(op_id, OpPhase::Intent, 1763030400, None),
        values: pairs(values),
        remote: remote.map(str::to_string),
    }
}

fn closed(op_id: &str, phase: OpPhase, result: OpResult) -> MarkedRecord {
    MarkedRecord { record: record(op_id, phase, 1763030500, Some(result)), values: vec![], remote: None }
}

fn seeded(log: &[MarkedRecord]) -> Fixture {
    let f = Fixture::new();
    for m in log {
        append_marked_record(&f.common, &m.record, &m.values, m.remote.as_deref()).unwrap();
    }
    f
}

/// close_intent on the seeded `log`: the record it returns equals `expected` but for `at`, which lies
/// between clock reads around the call and is taken into `expected`; the log gains exactly that record.
fn assert_closes(f: &Fixture, log: &[MarkedRecord], op_id: &str, close: IntentClose, mut expected: MarkedRecord) {
    let before = unix_now();
    let got = close_intent(&f.common, op_id, close).unwrap();
    let after = unix_now();
    assert!((before..=after).contains(&got.record.at), "at {} not in {before}..={after}", got.record.at);
    expected.record.at = got.record.at;
    assert_eq!(got, expected);
    let read = read_marked_log(&f.common).unwrap();
    assert!(read.torn_lines.is_empty());
    let mut all = log.to_vec();
    all.push(expected);
    assert_eq!(read.records, all);
}

/// close_intent refuses `op_id` as NotOpen naming it, and the log's bytes stay as they were.
fn assert_not_open(f: &Fixture, op_id: &str, close: IntentClose) {
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    match close_intent(&f.common, op_id, close) {
        Err(OplogError::NotOpen { op_id: named }) => assert_eq!(named, op_id),
        other => panic!("expected NotOpen for {op_id}: {other:?}"),
    }
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}

#[test]
fn done_on_an_open_marked_intent_keeps_record_values_and_mark() {
    let log = [intent("op-7", &[("b", "2"), ("a", "1")], Some("origin"))];
    let expected = MarkedRecord {
        record: record("op-7", OpPhase::Done, 0, Some(OpResult::Moved)),
        values: pairs(&[("b", "2"), ("a", "1")]),
        remote: Some("origin".to_string()),
    };
    assert_closes(&seeded(&log), &log, "op-7", IntentClose::Done, expected);
}

#[test]
fn failed_carries_the_refs_in_order_including_absent_ones() {
    let log = [intent("op-7", &[], None)];
    let refs = vec![
        ("refs/heads/main".to_string(), Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"))),
        ("refs/heads/gone".to_string(), None),
    ];
    let expected_refs = vec![
        ("refs/heads/main".to_string(), Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"))),
        ("refs/heads/gone".to_string(), None),
    ];
    let expected = MarkedRecord {
        record: record("op-7", OpPhase::Failed, 0, Some(OpResult::NotMoved { refs: expected_refs })),
        values: vec![],
        remote: None,
    };
    assert_closes(&seeded(&log), &log, "op-7", IntentClose::Failed { refs }, expected);
}

#[test]
fn an_unknown_op_id_is_not_open() {
    let f = seeded(&[intent("op-7", &[], Some("origin"))]);
    assert_not_open(&f, "op-9", IntentClose::Done);
}

#[test]
fn an_intent_already_closed_done_is_not_open() {
    let f = seeded(&[intent("op-7", &[], Some("origin")), closed("op-7", OpPhase::Done, OpResult::Moved)]);
    assert_not_open(&f, "op-7", IntentClose::Done);
}

#[test]
fn an_intent_already_closed_failed_is_not_open() {
    let f = seeded(&[
        intent("op-7", &[], None),
        closed("op-7", OpPhase::Failed, OpResult::NotMoved { refs: vec![] }),
    ]);
    assert_not_open(&f, "op-7", IntentClose::Failed { refs: vec![] });
}

#[test]
fn picks_the_intent_with_the_op_id_among_several_open_ones() {
    let log = [
        intent("op-1", &[("n", "1")], None),
        intent("op-2", &[("n", "2")], Some("origin")),
        intent("op-3", &[("n", "3")], Some("backup")),
    ];
    let expected = MarkedRecord {
        record: record("op-2", OpPhase::Done, 0, Some(OpResult::Moved)),
        values: pairs(&[("n", "2")]),
        remote: Some("origin".to_string()),
    };
    assert_closes(&seeded(&log), &log, "op-2", IntentClose::Done, expected);
}
