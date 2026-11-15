//! matching_records: every filter must keep a record (AND), log order kept, no filter keeps all. ZUT: oplog's
//! OpRecord, RefMove and HistoryFilter and gitio's Oid::parse are used as plain data.

use gitio::Oid;
use oplog::{HistoryFilter, OpPhase, OpRecord, RefMove};

use super::matching_records;

fn record(op_id: &str, key: &str, actor: &str, refs: &[&str]) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: key.to_string(),
        kind: "save".to_string(),
        actor: actor.to_string(),
        reason: "r".to_string(),
        at: 1,
        phase: OpPhase::Done,
        moves: refs
            .iter()
            .map(|refname| RefMove {
                refname: refname.to_string(),
                old: None,
                new: Oid::parse(&"b".repeat(40)).expect("a valid oid"),
            })
            .collect(),
        result: None,
    }
}

fn log() -> Vec<OpRecord> {
    vec![
        record("op-1", "k-1", "bob", &["refs/heads/zone/core"]),
        record("op-2", "k-2", "ann", &["refs/heads/zone/core", "refs/heads/req/core/a"]),
        record("op-3", "k-1", "ann", &["refs/heads/zone/core-old"]),
        record("op-4", "k-3", "bob", &[]),
    ]
}

fn ids(records: &[&OpRecord]) -> Vec<String> {
    records.iter().map(|record| record.op_id.clone()).collect()
}

fn ref_filter(refname: &str) -> HistoryFilter {
    HistoryFilter::Ref(refname.to_string())
}

#[test]
fn no_filter_keeps_every_record_in_log_order() {
    assert_eq!(ids(&matching_records(&log(), &[])), vec!["op-1", "op-2", "op-3", "op-4"]);
}

#[test]
fn ref_keeps_any_move_of_that_ref_matched_whole() {
    assert_eq!(ids(&matching_records(&log(), &[ref_filter("refs/heads/zone/core")])), vec!["op-1", "op-2"]);
    assert_eq!(ids(&matching_records(&log(), &[ref_filter("refs/heads/req/core/a")])), vec!["op-2"]);
}

#[test]
fn actor_and_key_match_exactly() {
    let log = log();
    assert_eq!(ids(&matching_records(&log, &[HistoryFilter::Actor("ann".to_string())])), vec!["op-2", "op-3"]);
    assert_eq!(ids(&matching_records(&log, &[HistoryFilter::Key("k-1".to_string())])), vec!["op-1", "op-3"]);
    assert!(matching_records(&log, &[HistoryFilter::Actor("an".to_string())]).is_empty());
}

#[test]
fn several_filters_are_anded() {
    let log = log();
    let core = ref_filter("refs/heads/zone/core");
    assert_eq!(ids(&matching_records(&log, &[core.clone(), HistoryFilter::Actor("ann".to_string())])), vec!["op-2"]);
    let key_and_actor = [HistoryFilter::Key("k-1".to_string()), HistoryFilter::Actor("ann".to_string())];
    assert_eq!(ids(&matching_records(&log, &key_and_actor)), vec!["op-3"]);
    assert!(matching_records(&log, &[core, HistoryFilter::Key("k-3".to_string())]).is_empty());
}
