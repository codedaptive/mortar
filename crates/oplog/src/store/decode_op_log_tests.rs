//! decode_op_log: records in order, torn lines skipped and numbered, any other bad line refused by number.

use super::{decode_op_log, OpLog, OpPhase, OpRecord, OpResult, OplogError};

const L1: &str = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[]}"#;
const L2: &str = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":11,"phase":"done","moves":[],"result":{"outcome":"moved"}}"#;
const TORN: &str = r#"{"op_id":"2","key":"k2","ki"#;

fn r1() -> OpRecord {
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

fn r2() -> OpRecord {
    OpRecord {
        at: 11,
        phase: OpPhase::Done,
        result: Some(OpResult::Moved),
        ..r1()
    }
}

fn log(records: Vec<OpRecord>, torn_lines: Vec<usize>) -> OpLog {
    OpLog { records, torn_lines }
}

fn malformed(bytes: &[u8]) -> (Option<usize>, String) {
    match decode_op_log(bytes) {
        Err(OplogError::Malformed { line, reason }) => (line, reason),
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn empty_file_is_an_empty_log() {
    assert_eq!(decode_op_log(b"").unwrap(), log(Vec::new(), Vec::new()));
}

#[test]
fn records_in_order() {
    let text = format!("{L1}\n{L2}\n");
    assert_eq!(decode_op_log(text.as_bytes()).unwrap(), log(vec![r1(), r2()], Vec::new()));
}

#[test]
fn last_line_without_newline_that_decodes_is_kept() {
    let text = format!("{L1}\n{L2}");
    assert_eq!(decode_op_log(text.as_bytes()).unwrap(), log(vec![r1(), r2()], Vec::new()));
}

#[test]
fn torn_tail_is_skipped_and_numbered() {
    let text = format!("{L1}\n{TORN}");
    assert_eq!(decode_op_log(text.as_bytes()).unwrap(), log(vec![r1()], vec![2]));
}

#[test]
fn torn_middle_line_is_skipped_and_numbered() {
    let text = format!("{TORN}\n{L1}\n{TORN}\n{L2}\n");
    assert_eq!(decode_op_log(text.as_bytes()).unwrap(), log(vec![r1(), r2()], vec![1, 3]));
}

#[test]
fn blank_lines_are_skipped_but_counted() {
    let text = format!("\n\n{L1}\n\n{TORN}\n\n{L2}\n\n");
    assert_eq!(decode_op_log(text.as_bytes()).unwrap(), log(vec![r1(), r2()], vec![5]));
}

#[test]
fn utf8_cut_short_at_the_line_end_is_torn() {
    let mut bytes = format!("{L1}\n").into_bytes();
    bytes.extend_from_slice(b"{\"op_id\":\"\xe4\xb8");
    bytes.extend_from_slice(format!("\n{L2}\n").as_bytes());
    assert_eq!(decode_op_log(&bytes).unwrap(), log(vec![r1(), r2()], vec![2]));
}

#[test]
fn undecodable_middle_line_is_malformed_with_its_number() {
    let text = format!("{L1}\n\n{{\"op_id\":1}}\n{L2}\n");
    assert_eq!(
        malformed(text.as_bytes()),
        (Some(3), "op_id is not a string".to_string())
    );
}

#[test]
fn bad_utf8_inside_a_line_is_malformed() {
    let mut bytes = format!("{L1}\n").into_bytes();
    bytes.extend_from_slice(b"{\xff}\n");
    assert_eq!(malformed(&bytes), (Some(2), "not UTF-8 at byte 1".to_string()));
}

#[test]
fn complete_json_with_trailing_text_is_malformed_not_torn() {
    let text = format!("{L1}{L2}\n");
    let (line, reason) = malformed(text.as_bytes());
    assert_eq!(line, Some(1));
    assert!(reason.contains("after the value"), "{reason}");
}
