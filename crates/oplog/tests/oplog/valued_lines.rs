//! Joins the valued op log line's encoder and decoder with the log file: append_valued_record ->
//! append_marked_record -> log_line -> encode_marked_record writes the exact bytes into a throwaway
//! repository's op log after an empty file, a '\n'-ended file and a torn tail, and decode_valued_log ->
//! decode_marked_log -> decode_marked_line, then unmarked_log, reads one-line logs back: values, no
//! values, refusals, torn prefixes and encode_valued_record's lines.

use crate::support::Fixture;
use gitio::Oid;
use oplog::{
    append_valued_record, decode_valued_log, encode_valued_record, oplog_path, OpPhase, OpRecord,
    OpResult, OplogError, RefMove, ValuedLog, ValuedRecord,
};
use std::fs;

const WITH_VALUES: &str = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"done","moves":[],"result":{"outcome":"moved"},"values":[{"name":"t","value":"1"},{"name":"t","value":"é\n"}]}"#;

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

/// The record the literal lines below carry.
fn simms() -> OpRecord {
    OpRecord {
        actor: "simms".to_string(),
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
    append_valued_record(&f.common, &simms(), values).unwrap();
    fs::read_to_string(&path).unwrap()
}

#[test]
fn empty_file_gets_the_record_and_a_newline() {
    assert_eq!(
        written(None, &[]),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn file_ending_in_newline_gets_no_leading_newline() {
    let seed = format!("{EARLIER}\n");
    assert_eq!(
        written(Some(&seed), &[]),
        seed + "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn torn_tail_gets_a_newline_before_the_record() {
    assert_eq!(
        written(Some(EARLIER), &[]),
        EARLIER.to_string()
            + "\n{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
}

#[test]
fn values_go_last_on_the_line() {
    let seed = format!("{EARLIER}\n");
    assert_eq!(
        written(Some(&seed), &pairs(&[("folder_tree", "t1")])),
        seed + "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"folder_tree\",\"value\":\"t1\"}]}\n"
    );
}
