//! recovered_records: every record kept in log order, then the closing of each open intent not marked with a remote,
//! in log order, appended last; an intent closed later in the log, a remote-marked intent and a None closing add
//! nothing; close's Err is the answer.

use std::cell::RefCell;

use gitio::Oid;
use oplog::{MarkedRecord, OpPhase, OpRecord, RefMove, ValuedRecord};

use super::recovered_records;

const REQUEST: &str = "refs/heads/req/core/fix-1";
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn op(op_id: &str, phase: OpPhase, at: u64) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("join-rebase:{op_id}"),
        kind: "join-rebase".to_string(),
        actor: "agent-a".to_string(),
        reason: "test".to_string(),
        at,
        phase,
        moves: vec![RefMove {
            refname: REQUEST.to_string(),
            old: Some(Oid::parse(OLD).unwrap()),
            new: Oid::parse(NEW).unwrap(),
        }],
        result: None,
    }
}

fn values(op_id: &str) -> Vec<(String, String)> {
    vec![("folder_tree".to_string(), format!("tree-of-{op_id}"))]
}

fn marked(op_id: &str, phase: OpPhase, remote: Option<&str>) -> MarkedRecord {
    MarkedRecord { record: op(op_id, phase, 1_700_000_000), values: values(op_id), remote: remote.map(str::to_string) }
}

fn valued(op_id: &str, phase: OpPhase) -> ValuedRecord {
    ValuedRecord { record: op(op_id, phase, 1_700_000_000), values: values(op_id) }
}

/// The literal closing the fake gives `op_id`: a Done record at a later time.
fn closing(op_id: &str) -> ValuedRecord {
    ValuedRecord { record: op(op_id, OpPhase::Done, 1_700_000_999), values: values(op_id) }
}

/// Runs recovered_records with a fake close that records each op_id it is asked for and closes every intent except
/// those in `keep_open`.
fn run(records: &[MarkedRecord], keep_open: &[&str]) -> (Vec<ValuedRecord>, Vec<String>) {
    let asked = RefCell::new(Vec::new());
    let out = recovered_records(records, |intent: &MarkedRecord| -> Result<Option<ValuedRecord>, String> {
        let op_id = intent.record.op_id.clone();
        asked.borrow_mut().push(op_id.clone());
        Ok((!keep_open.contains(&op_id.as_str())).then(|| closing(&op_id)))
    })
    .unwrap();
    (out, asked.into_inner())
}

#[test]
fn no_intents_leaves_the_records_unchanged_and_closes_nothing() {
    let records = [marked("op-a", OpPhase::Done, None), marked("op-b", OpPhase::Failed, None)];
    let (out, asked) = run(&records, &[]);
    assert_eq!(out, vec![valued("op-a", OpPhase::Done), valued("op-b", OpPhase::Failed)]);
    assert!(asked.is_empty());
}

#[test]
fn an_empty_log_is_empty() {
    let (out, asked) = run(&[], &[]);
    assert!(out.is_empty());
    assert!(asked.is_empty());
}

#[test]
fn one_open_intent_is_closed_and_its_closing_appended_last() {
    let records = [marked("op-rebase", OpPhase::Intent, None), marked("op-save", OpPhase::Done, None)];
    let (out, asked) = run(&records, &[]);
    assert_eq!(asked, vec!["op-rebase".to_string()]);
    assert_eq!(
        out,
        vec![valued("op-rebase", OpPhase::Intent), valued("op-save", OpPhase::Done), closing("op-rebase")]
    );
}

#[test]
fn an_intent_closed_later_in_the_log_is_not_passed_to_close() {
    for phase in [OpPhase::Done, OpPhase::Failed] {
        let records = [marked("op-a", OpPhase::Intent, None), marked("op-a", phase, None)];
        let (out, asked) = run(&records, &[]);
        assert!(asked.is_empty());
        assert_eq!(out, vec![valued("op-a", OpPhase::Intent), valued("op-a", phase)]);
    }
}

#[test]
fn a_close_before_the_intent_does_not_close_it() {
    let records = [marked("op-a", OpPhase::Done, None), marked("op-a", OpPhase::Intent, None)];
    let (out, asked) = run(&records, &[]);
    assert_eq!(asked, vec!["op-a".to_string()]);
    assert_eq!(out.len(), 3);
    assert_eq!(out[2], closing("op-a"));
}

#[test]
fn a_remote_marked_intent_is_never_passed_to_close() {
    let records = [marked("op-push", OpPhase::Intent, Some("origin")), marked("op-rebase", OpPhase::Intent, None)];
    let (out, asked) = run(&records, &[]);
    assert_eq!(asked, vec!["op-rebase".to_string()]);
    assert_eq!(
        out,
        vec![valued("op-push", OpPhase::Intent), valued("op-rebase", OpPhase::Intent), closing("op-rebase")]
    );
}

#[test]
fn a_none_closing_leaves_the_intent_open() {
    let records = [marked("op-mixed", OpPhase::Intent, None)];
    let (out, asked) = run(&records, &["op-mixed"]);
    assert_eq!(asked, vec!["op-mixed".to_string()]);
    assert_eq!(out, vec![valued("op-mixed", OpPhase::Intent)]);
}

#[test]
fn two_open_intents_are_closed_in_log_order_after_all_the_records() {
    let records = [
        marked("op-first", OpPhase::Intent, None),
        marked("op-done", OpPhase::Done, None),
        marked("op-second", OpPhase::Intent, None),
    ];
    let (out, asked) = run(&records, &[]);
    assert_eq!(asked, vec!["op-first".to_string(), "op-second".to_string()]);
    assert_eq!(
        out,
        vec![
            valued("op-first", OpPhase::Intent),
            valued("op-done", OpPhase::Done),
            valued("op-second", OpPhase::Intent),
            closing("op-first"),
            closing("op-second"),
        ]
    );
}

#[test]
fn an_err_from_close_is_returned() {
    let records = [marked("op-rebase", OpPhase::Intent, None)];
    let out = recovered_records(&records, |_: &MarkedRecord| -> Result<Option<ValuedRecord>, String> {
        Err("ref unreadable".to_string())
    });
    assert_eq!(out, Err("ref unreadable".to_string()));
}
