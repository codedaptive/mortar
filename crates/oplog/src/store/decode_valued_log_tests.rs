//! decode_valued_log: lines with and without values read whole and in order, values kept per record,
//! torn lines numbered, any other bad line refused by number; one-line logs (values, no values, refusals,
//! every torn prefix), a marked line with its mark dropped, and encode_valued_record's lines read back. It runs
//! decode_marked_log, decode_marked_line, unmarked_log and encode_valued_record for real: all pure, so these are
//! unit tests.

use gitio::Oid;

use super::{
    decode_valued_log, encode_valued_record, OpPhase, OpRecord, OpResult, OplogError, RefMove, ValuedLog,
    ValuedRecord,
};

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

const WITH_VALUES: &str = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"done","moves":[],"result":{"outcome":"moved"},"values":[{"name":"t","value":"1"},{"name":"t","value":"é\n"}]}"#;

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

/// `line` as a one-line log through decode_valued_log.
fn one_line(line: &str) -> Result<ValuedLog, OplogError> {
    decode_valued_log(line.as_bytes())
}

fn found(record: ValuedRecord) -> ValuedLog {
    ValuedLog {
        records: vec![record],
        torn_lines: Vec::new(),
    }
}

fn malformed_reason(got: Result<ValuedLog, OplogError>) -> (Option<usize>, String) {
    match got {
        Err(OplogError::Malformed { line, reason }) => (line, reason),
        other => panic!("expected Malformed, got {other:?}"),
    }
}

#[test]
fn a_line_with_values_gives_the_record_and_its_values() {
    assert_eq!(
        one_line(WITH_VALUES).unwrap(),
        found(ValuedRecord {
            record: record(OpPhase::Done, Some(OpResult::Moved)),
            values: pairs(&[("t", "1"), ("t", "é\n")]),
        })
    );
}

#[test]
fn a_line_without_the_key_has_no_values() {
    let line = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"intent","moves":[]}"#;
    assert_eq!(
        one_line(line).unwrap(),
        found(ValuedRecord {
            record: record(OpPhase::Intent, None),
            values: Vec::new(),
        })
    );
}

#[test]
fn a_bad_values_member_is_malformed() {
    let line = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"intent","moves":[],"values":[{"name":"t"}]}"#;
    assert_eq!(
        malformed_reason(one_line(line)),
        (Some(1), "value 1: missing key \"value\"".to_string())
    );
    let twice = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"intent","moves":[],"values":[],"values":[]}"#;
    assert_eq!(
        malformed_reason(one_line(twice)),
        (Some(1), "the record has key \"values\" twice".to_string())
    );
}

#[test]
fn every_non_empty_proper_prefix_is_torn() {
    let mut count = 0;
    for (cut, _) in WITH_VALUES.char_indices() {
        let prefix = &WITH_VALUES[..cut];
        // decode_valued_log skips an empty line before decoding it, so the empty prefix reads as no line.
        let expected = if prefix.is_empty() {
            ValuedLog::default()
        } else {
            ValuedLog {
                records: Vec::new(),
                torn_lines: vec![1],
            }
        };
        assert_eq!(one_line(prefix).unwrap(), expected, "{prefix:?}");
        count += 1;
    }
    assert_eq!(count, WITH_VALUES.chars().count());
}

#[test]
fn encoded_records_read_back() {
    let cases = vec![
        (record(OpPhase::Intent, None), Vec::new()),
        (record(OpPhase::Intent, None), pairs(&[("", "")])),
        (
            OpRecord {
                moves: vec![RefMove {
                    refname: "refs/heads/\"x\"".to_string(),
                    old: None,
                    new: Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap(),
                }],
                ..record(OpPhase::Done, Some(OpResult::Moved))
            },
            pairs(&[("folder_tree", "abc"), ("q\"\\", "\u{1}\t主")]),
        ),
        (
            record(OpPhase::Failed, Some(OpResult::NotMoved { refs: Vec::new() })),
            pairs(&[("a", "1"), ("a", "1"), ("b", "")]),
        ),
    ];
    for (r, values) in cases {
        let line = encode_valued_record(&r, &values);
        assert_eq!(
            one_line(&line).unwrap(),
            found(ValuedRecord { record: r, values }),
            "{line}"
        );
    }
}

#[test]
fn a_marked_line_decodes_with_the_mark_dropped() {
    let line = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"done","moves":[],"result":{"outcome":"moved"},"values":[{"name":"t","value":"1"},{"name":"t","value":"é\n"}],"remote":"origin"}"#;
    assert_eq!(
        decode_valued_log(line.as_bytes()).unwrap(),
        ValuedLog {
            records: vec![ValuedRecord {
                record: record(OpPhase::Done, Some(OpResult::Moved)),
                values: pairs(&[("t", "1"), ("t", "é\n")]),
            }],
            torn_lines: Vec::new(),
        }
    );
}
