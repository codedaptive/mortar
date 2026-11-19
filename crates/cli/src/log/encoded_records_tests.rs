//! encoded_records: no lines without --json; with it, each record's op log line (record, values, remote mark) in the
//! order given. Unit tests: oplog's record types and gitio's Oid::parse are plain data; the expected lines are literals
//! written by hand in oplog's key order (the record's keys, then "values", then "remote" last).

use gitio::Oid;
use oplog::{MarkedRecord, OpPhase, OpRecord, RefMove};

use super::encoded_records;

fn record(op_id: &str) -> MarkedRecord {
    let record = OpRecord {
        op_id: op_id.to_string(),
        key: "k".to_string(),
        kind: "save".to_string(),
        actor: "core/fix-1".to_string(),
        reason: "mr save".to_string(),
        at: 5,
        phase: OpPhase::Intent,
        moves: vec![RefMove {
            refname: "refs/heads/req/core/fix-1".to_string(),
            old: None,
            new: Oid::parse(&"c".repeat(40)).expect("a valid oid"),
        }],
        result: None,
    };
    MarkedRecord { record, values: Vec::new(), remote: None }
}

const OP_1: &str = r#"{"op_id":"op-1","key":"k","kind":"save","actor":"core/fix-1","reason":"mr save","at":5,"phase":"intent","moves":[{"ref":"refs/heads/req/core/fix-1","old":null,"new":"cccccccccccccccccccccccccccccccccccccccc"}]}"#;
const OP_2: &str = r#"{"op_id":"op-2","key":"k","kind":"save","actor":"core/fix-1","reason":"mr save","at":5,"phase":"intent","moves":[{"ref":"refs/heads/req/core/fix-1","old":null,"new":"cccccccccccccccccccccccccccccccccccccccc"}]}"#;
const OP_3_MARKED: &str = r#"{"op_id":"op-3","key":"k","kind":"save","actor":"core/fix-1","reason":"mr save","at":5,"phase":"intent","moves":[{"ref":"refs/heads/req/core/fix-1","old":null,"new":"cccccccccccccccccccccccccccccccccccccccc"}],"values":[{"name":"folder_tree","value":"t1"},{"name":"n","value":"v"}],"remote":"origin"}"#;

#[test]
fn without_json_two_records_give_no_lines() {
    let (one, two) = (record("op-1"), record("op-2"));
    assert_eq!(encoded_records(&[&one, &two], false), Vec::<String>::new());
}

#[test]
fn with_json_each_record_is_its_op_log_line() {
    let one = record("op-1");
    assert_eq!(encoded_records(&[&one], true), vec![OP_1.to_string()]);
}

#[test]
fn with_json_the_order_given_is_kept() {
    let (one, two) = (record("op-1"), record("op-2"));
    assert_eq!(encoded_records(&[&two, &one], true), vec![OP_2.to_string(), OP_1.to_string()]);
}

#[test]
fn with_json_a_records_values_and_remote_mark_are_in_its_line() {
    let marked = MarkedRecord {
        values: vec![("folder_tree".to_string(), "t1".to_string()), ("n".to_string(), "v".to_string())],
        remote: Some("origin".to_string()),
        ..record("op-3")
    };
    assert_eq!(encoded_records(&[&marked], true), vec![OP_3_MARKED.to_string()]);
}
