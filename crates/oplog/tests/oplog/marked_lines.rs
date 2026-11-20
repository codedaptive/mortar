//! Joins the op log line's encoder with the log file: append_valued_record and open_remote_intent ->
//! append_marked_record -> log_line -> encode_marked_record write the exact bytes into a throwaway repository's
//! op log, after a torn tail too. The decoder's and encoder's own decisions, and decode_valued_log reading a
//! marked line back with its mark dropped, are unit tests beside them in src/store/.

use crate::support::Fixture;
use oplog::{
    append_valued_record, open_remote_intent, oplog_path, Op, OpPhase, OpRecord, OpResult,
};
use std::fs;

fn record(
    op_id: &str,
    key: &str,
    kind: &str,
    actor: &str,
    at: u64,
    phase: OpPhase,
    result: Option<OpResult>,
) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: key.to_string(),
        kind: kind.to_string(),
        actor: actor.to_string(),
        reason: "r".to_string(),
        at,
        phase,
        moves: Vec::new(),
        result,
    }
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

/// The whole op log after one append_valued_record (no mark) into a fresh repository.
fn written(r: &OpRecord, values: &[(String, String)]) -> String {
    let f = Fixture::new();
    append_valued_record(&f.common, r, values).unwrap();
    fs::read_to_string(oplog_path(&f.common)).unwrap()
}

/// The whole op log after open_remote_intent, in a fresh repository, opens an intent with push_at_7's fields,
/// marked `remote` and with no values: the line append_marked_record writes for push_at_7, no values, `remote`.
fn opened_remote(remote: &str) -> String {
    let f = Fixture::new();
    let op = Op {
        op_id: "o".to_string(),
        key: "k".to_string(),
        kind: "push".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 7,
        moves: Vec::new(),
    };
    open_remote_intent(&f.git, &f.common, &op, remote).unwrap();
    fs::read_to_string(oplog_path(&f.common)).unwrap()
}

/// The record the lines below carry.
fn push_at_7() -> OpRecord {
    record("o", "k", "push", "a", 7, OpPhase::Intent, None)
}

#[test]
fn no_mark_writes_no_remote_key() {
    assert_eq!(
        written(&push_at_7(), &[]),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
    assert_eq!(
        written(&push_at_7(), &pairs(&[("n", "v")])),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"n\",\"value\":\"v\"}]}\n"
    );
}

#[test]
fn a_mark_without_values_is_the_last_key() {
    assert_eq!(
        opened_remote("origin"),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"remote\":\"origin\"}\n"
    );
}

#[test]
fn a_name_needing_escapes_is_escaped() {
    assert_eq!(
        opened_remote("up\"st\\re\nam"),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"remote\":\"up\\\"st\\\\re\\nam\"}\n"
    );
}

#[test]
fn values_go_on_a_new_line_after_a_torn_tail() {
    let f = Fixture::new();
    let path = oplog_path(&f.common);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{\"op_id\":\"t\",\"key\":x").unwrap();
    let r = record("o", "k", "land", "agent-a", 5, OpPhase::Intent, None);
    append_valued_record(&f.common, &r, &pairs(&[("folder_tree", "t1")])).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "{\"op_id\":\"t\",\"key\":x\
         \n{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"agent-a\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"folder_tree\",\"value\":\"t1\"}]}\n"
    );
}
