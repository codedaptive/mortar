//! Joins the valued op log line's encoder with the log file: append_valued_record -> append_marked_record
//! -> log_line -> encode_marked_record writes the exact bytes into a throwaway repository's op log after an
//! empty file, a '\n'-ended file and a torn tail, with and without values.

use crate::support::Fixture;
use oplog::{append_valued_record, oplog_path, OpPhase, OpRecord, OpResult};
use std::fs;

/// A whole earlier line, without its '\n'.
const EARLIER: &str = r#"{"op_id":"e","key":"k0","kind":"land","actor":"a","reason":"r","at":1,"phase":"intent","moves":[]}"#;

fn record(phase: OpPhase, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "o".to_string(),
        key: "k".to_string(),
        kind: "land".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 3,
        phase,
        moves: Vec::new(),
        result,
    }
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

/// The record the literal lines below carry.
fn agent_a() -> OpRecord {
    OpRecord {
        actor: "agent-a".to_string(),
        at: 5,
        ..record(OpPhase::Intent, None)
    }
}

/// The whole op log after one append_valued_record into a fresh repository whose log starts as `seed`
/// (None: no log file yet).
fn written(seed: Option<&str>, values: &[(String, String)]) -> String {
    let f = Fixture::new();
    let path = oplog_path(&f.common);
    if let Some(seed) = seed {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, seed).unwrap();
    }
    append_valued_record(&f.common, &agent_a(), values).unwrap();
    fs::read_to_string(&path).unwrap()
}

#[test]
fn empty_file_gets_the_record_and_a_newline() {
    assert_eq!(
        written(None, &[]),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"agent-a\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn file_ending_in_newline_gets_no_leading_newline() {
    let seed = format!("{EARLIER}\n");
    assert_eq!(
        written(Some(&seed), &[]),
        seed + "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"agent-a\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn torn_tail_gets_a_newline_before_the_record() {
    assert_eq!(
        written(Some(EARLIER), &[]),
        EARLIER.to_string()
            + "\n{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"agent-a\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn values_go_last_on_the_line() {
    let seed = format!("{EARLIER}\n");
    assert_eq!(
        written(Some(&seed), &pairs(&[("folder_tree", "t1")])),
        seed + "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"agent-a\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"folder_tree\",\"value\":\"t1\"}]}\n"
    );
}
