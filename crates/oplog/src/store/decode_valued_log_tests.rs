//! decode_valued_log: lines with and without values read whole and in order, values kept per record,
//! torn lines numbered, any other bad line refused by number.

use super::{decode_valued_log, OpPhase, OpRecord, OpResult, OplogError, ValuedLog, ValuedRecord};

const PLAIN: &str = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[]}"#;
const VALUED: &str = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":11,"phase":"done","moves":[],"result":{"outcome":"moved"},"values":[{"name":"folder_tree","value":"t1"},{"name":"n","value":""}]}"#;
const TORN: &str = r#"{"op_id":"2","key":"k2","ki"#;

fn intent() -> OpRecord {
    OpRecord {
        op_id: "1".to_string(),
        key: "k1".to_string(),
        kind: "land".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 10,
        phase: OpPhase::Intent,
        moves: Vec::new(),
        result: None,
    }
}

fn plain() -> ValuedRecord {
    ValuedRecord {
        record: intent(),
        values: Vec::new(),
    }
}

fn valued() -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            at: 11,
            phase: OpPhase::Done,
            result: Some(OpResult::Moved),
            ..intent()
        },
        values: vec![
            ("folder_tree".to_string(), "t1".to_string()),
            ("n".to_string(), String::new()),
        ],
    }
}

#[test]
fn a_mixed_log_reads_whole_in_order_with_values_per_record() {
    let text = format!("{PLAIN}\n{VALUED}\n{PLAIN}\n");
    assert_eq!(
        decode_valued_log(text.as_bytes()).unwrap(),
        ValuedLog {
            records: vec![plain(), valued(), plain()],
            torn_lines: Vec::new(),
        }
    );
}

#[test]
fn a_torn_line_is_kept_by_number() {
    let text = format!("{VALUED}\n{TORN}\n\n{PLAIN}");
    assert_eq!(
        decode_valued_log(text.as_bytes()).unwrap(),
        ValuedLog {
            records: vec![valued(), plain()],
            torn_lines: vec![2],
        }
    );
}

#[test]
fn a_bad_values_member_is_malformed_with_its_line() {
    let bad = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[],"values":"x"}"#;
    let text = format!("{PLAIN}\n{bad}\n");
    match decode_valued_log(text.as_bytes()) {
        Err(OplogError::Malformed { line, reason }) => {
            assert_eq!((line, reason.as_str()), (Some(2), "values is not an array"));
        }
        other => panic!("expected Malformed, got {other:?}"),
    }
}
