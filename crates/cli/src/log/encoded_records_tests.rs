//! encoded_records: no lines without --json; with it, encode_op_record's line for each record in the order given. ZUT:
//! oplog's record types and gitio's Oid::parse are plain data; the expected lines are literals written by hand.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove};

use super::encoded_records;

fn record(op_id: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: "k".to_string(),
        kind: "save".to_string(),
        actor: "core/fix-1".to_string(),
        reason: "mortar save".to_string(),
        at: 5,
        phase: OpPhase::Intent,
        moves: vec![RefMove {
            refname: "refs/heads/req/core/fix-1".to_string(),
            old: None,
            new: Oid::parse(&"c".repeat(40)).expect("a valid oid"),
        }],
        result: None,
    }
}

const OP_1: &str = r#"{"op_id":"op-1","key":"k","kind":"save","actor":"core/fix-1","reason":"mortar save","at":5,"phase":"intent","moves":[{"ref":"refs/heads/req/core/fix-1","old":null,"new":"cccccccccccccccccccccccccccccccccccccccc"}]}"#;
const OP_2: &str = r#"{"op_id":"op-2","key":"k","kind":"save","actor":"core/fix-1","reason":"mortar save","at":5,"phase":"intent","moves":[{"ref":"refs/heads/req/core/fix-1","old":null,"new":"cccccccccccccccccccccccccccccccccccccccc"}]}"#;

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
