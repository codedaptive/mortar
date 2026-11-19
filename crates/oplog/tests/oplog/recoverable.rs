//! Joins recover -> read_marked_log -> recoverable_intents (open_intents_in) -> decide_recovery ->
//! recovery_close_valued -> append_valued_record on a log seeded with encode_marked_record lines: open unmarked
//! intents are settled with their values in log order; marked and closed intents get nothing appended.
//! Every seeded intent has no moves, so decide_recovery closes each one done/Moved; the closing record's
//! `at` is the clock (unix_now), so it is checked against reads before and after.

use crate::support::Fixture;
use oplog::{
    encode_marked_record, oplog_path, read_marked_log, read_valued_log, recover, unix_now, MarkedRecord, OpPhase,
    OpRecord, OpResult,
};
use std::fs;

fn record(op_id: &str, phase: OpPhase, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key:{op_id}"),
        kind: "publish".to_string(),
        actor: "agent-a".to_string(),
        reason: "squash publishes".to_string(),
        at: 1763030400,
        phase,
        moves: vec![],
        result,
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

fn intent(op_id: &str, values: &[(&str, &str)], remote: Option<&str>) -> MarkedRecord {
    MarkedRecord {
        record: record(op_id, OpPhase::Intent, None),
        values: pairs(values),
        remote: remote.map(str::to_string),
    }
}

fn closing(op_id: &str, phase: OpPhase, result: OpResult, remote: Option<&str>) -> MarkedRecord {
    MarkedRecord { record: record(op_id, phase, Some(result)), values: vec![], remote: remote.map(str::to_string) }
}

/// The done record recover appends for `op_id`'s intent, with `values` and no mark; `at` is filled from
/// the record recover returned.
fn done(op_id: &str, values: &[(&str, &str)], at: u64) -> MarkedRecord {
    MarkedRecord {
        record: OpRecord { at, ..record(op_id, OpPhase::Done, Some(OpResult::Moved)) },
        values: pairs(values),
        remote: None,
    }
}

/// A fixture whose log holds `log`, one encode_marked_record line each: the bytes append_marked_record appends,
/// written directly because no export appends an arbitrary marked record.
fn seeded(log: &[MarkedRecord]) -> Fixture {
    let f = Fixture::new();
    if !log.is_empty() {
        let path = oplog_path(&f.common);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text: String =
            log.iter().map(|m| encode_marked_record(&m.record, &m.values, m.remote.as_deref()) + "\n").collect();
        fs::write(&path, text).unwrap();
    }
    f
}

/// recover on the seeded `log` closes exactly `settled` (op_id and values), done, in this order: its
/// returned records are those, each `at` between clock reads around the call, and the log is `log` with
/// those records appended, values carried, nothing for any other intent.
fn assert_recovers(log: &[MarkedRecord], settled: &[(&str, &[(&str, &str)])]) {
    let f = seeded(log);
    let before = unix_now();
    let returned = recover(&f.git, &f.common).unwrap();
    let after = unix_now();
    assert_eq!(returned.len(), settled.len());
    let ats: Vec<u64> = returned.iter().map(|r| r.at).collect();
    assert!(ats.iter().all(|at| (before..=after).contains(at)), "{ats:?} not in {before}..={after}");
    let expected: Vec<MarkedRecord> =
        settled.iter().zip(&ats).map(|((id, values), &at)| done(id, values, at)).collect();
    assert_eq!(returned, expected.iter().map(|m| m.record.clone()).collect::<Vec<_>>());
    let marked = read_marked_log(&f.common).unwrap();
    assert!(marked.torn_lines.is_empty());
    let mut all = log.to_vec();
    all.extend(expected.iter().cloned());
    assert_eq!(marked.records, all);
    let valued = read_valued_log(&f.common).unwrap();
    let closing_values: Vec<_> = valued.records[log.len()..].iter().map(|v| v.values.clone()).collect();
    assert_eq!(closing_values, expected.iter().map(|m| m.values.clone()).collect::<Vec<_>>());
}

#[test]
fn an_open_marked_intent_is_skipped_while_the_unmarked_one_beside_it_is_kept() {
    assert_recovers(&[intent("op-1", &[], Some("origin")), intent("op-2", &[], None)], &[("op-2", &[])]);
}

#[test]
fn intents_closed_by_a_later_done_or_failed_record_are_not_returned() {
    let log = [
        intent("op-1", &[], None),
        intent("op-2", &[], None),
        intent("op-3", &[], None),
        closing("op-1", OpPhase::Done, OpResult::Moved, None),
        closing("op-2", OpPhase::Failed, OpResult::NotMoved { refs: vec![] }, None),
    ];
    assert_recovers(&log, &[("op-3", &[])]);
}

#[test]
fn a_marked_intent_closed_by_a_later_record_is_not_returned() {
    let f = seeded(&[
        intent("op-1", &[], Some("origin")),
        closing("op-1", OpPhase::Done, OpResult::Moved, Some("origin")),
    ]);
    let bytes = fs::read(oplog_path(&f.common)).unwrap();
    assert_eq!(recover(&f.git, &f.common).unwrap(), vec![]);
    assert_eq!(fs::read(oplog_path(&f.common)).unwrap(), bytes);
}

#[test]
fn values_are_carried_in_their_order() {
    let values: &[(&str, &str)] = &[("folder_tree", "3f2a"), ("actor", "agent-a"), ("folder_tree", "")];
    assert_recovers(&[intent("op-1", values, None)], &[("op-1", values)]);
}

#[test]
fn log_order_is_kept() {
    let log = [
        intent("op-c", &[], None),
        intent("op-x", &[], Some("origin")),
        intent("op-a", &[], None),
        intent("op-b", &[("n", "1")], None),
    ];
    assert_recovers(&log, &[("op-c", &[]), ("op-a", &[]), ("op-b", &[("n", "1")])]);
}
