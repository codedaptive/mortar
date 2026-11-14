//! Joins the marked op log line's encoder and decoder with the log file: append_marked_record -> log_line
//! -> encode_marked_record writes the exact bytes into a throwaway repository's op log, and
//! decode_marked_log / decode_valued_log -> decode_marked_line read one-line logs back, marks, torn
//! prefixes and refusals included.

use crate::support::Fixture;
use gitio::Oid;
use oplog::{
    append_marked_record, decode_marked_log, decode_valued_log, oplog_path, MarkedLog, MarkedRecord,
    OpPhase, OpRecord, OpResult, OplogError, RefMove, ValuedLog, ValuedRecord,
};
use std::fs;

const MARKED: &str = r#"{"op_id":"o","key":"k","kind":"push","actor":"a","reason":"r","at":3,"phase":"intent","moves":[],"values":[{"name":"t","value":"1"}],"remote":"origin"}"#;
const PREFIX: &str = r#"{"op_id":"o","key":"k","kind":"push","actor":"a","reason":"r","at":3,"phase":"intent","moves":[]"#;

const OLD: &str = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[]}"#;
const MARKED_INTENT: &str = r#"{"op_id":"2","key":"k2","kind":"push","actor":"a","reason":"r","at":10,"phase":"intent","moves":[],"remote":"origin"}"#;
const MARKED_DONE: &str = r#"{"op_id":"2","key":"k2","kind":"push","actor":"a","reason":"r","at":12,"phase":"done","moves":[],"result":{"outcome":"moved"},"values":[{"name":"n","value":"v"}],"remote":"origin"}"#;
const TORN: &str = r#"{"op_id":"3","key":"k3","kind":"push","remo"#;

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

/// The record MARKED and PREFIX carry.
fn push(phase: OpPhase, result: Option<OpResult>) -> OpRecord {
    record("o", "k", "push", "a", 3, phase, result)
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

/// `line` as a one-line log through decode_marked_log.
fn one_line(line: &str) -> Result<MarkedLog, OplogError> {
    decode_marked_log(line.as_bytes())
}

fn found(record: MarkedRecord) -> MarkedLog {
    MarkedLog {
        records: vec![record],
        torn_lines: Vec::new(),
    }
}

fn malformed_reason(got: Result<MarkedLog, OplogError>) -> (Option<usize>, String) {
    match got {
        Err(OplogError::Malformed { line, reason }) => (line, reason),
        other => panic!("expected Malformed, got {other:?}"),
    }
}

/// The whole op log after one append_marked_record into a fresh repository.
fn written(r: &OpRecord, values: &[(String, String)], remote: Option<&str>) -> String {
    let f = Fixture::new();
    append_marked_record(&f.common, r, values, remote).unwrap();
    fs::read_to_string(oplog_path(&f.common)).unwrap()
}

#[test]
fn a_marked_line_gives_its_mark() {
    assert_eq!(
        one_line(MARKED).unwrap(),
        found(MarkedRecord {
            record: push(OpPhase::Intent, None),
            values: pairs(&[("t", "1")]),
            remote: Some("origin".to_string()),
        })
    );
}

#[test]
fn a_line_without_the_mark_has_none() {
    assert_eq!(
        one_line(&format!("{PREFIX}}}")).unwrap(),
        found(MarkedRecord {
            record: push(OpPhase::Intent, None),
            values: Vec::new(),
            remote: None,
        })
    );
}

#[test]
fn a_mark_twice_is_malformed() {
    let line = format!(r#"{PREFIX},"remote":"a","remote":"b"}}"#);
    assert_eq!(
        malformed_reason(one_line(&line)),
        (Some(1), "the record has key \"remote\" twice".to_string())
    );
}

#[test]
fn a_mark_that_is_not_a_string_is_malformed() {
    for value in ["1", "null", "[]", "{}"] {
        let line = format!(r#"{PREFIX},"remote":{value}}}"#);
        assert_eq!(
            malformed_reason(one_line(&line)),
            (Some(1), "remote is not a string".to_string()),
            "{line}"
        );
    }
}

#[test]
fn every_non_empty_proper_prefix_is_torn() {
    let mut count = 0;
    for (cut, _) in MARKED.char_indices() {
        let prefix = &MARKED[..cut];
        // decode_marked_log skips an empty line before decoding it, so the empty prefix reads as no line.
        let expected = if prefix.is_empty() {
            MarkedLog::default()
        } else {
            MarkedLog {
                records: Vec::new(),
                torn_lines: vec![1],
            }
        };
        assert_eq!(one_line(prefix).unwrap(), expected, "{prefix:?}");
        count += 1;
    }
    assert_eq!(count, MARKED.chars().count());
}

#[test]
fn encoded_records_read_back() {
    let moved = OpRecord {
        moves: vec![RefMove {
            refname: "refs/heads/x".to_string(),
            old: None,
            new: Oid::parse("cccccccccccccccccccccccccccccccccccccccc").unwrap(),
        }],
        ..push(OpPhase::Done, Some(OpResult::Moved))
    };
    let cases = vec![
        (push(OpPhase::Intent, None), Vec::new(), None),
        (push(OpPhase::Intent, None), Vec::new(), Some("origin")),
        (moved.clone(), pairs(&[("folder_tree", "abc")]), None),
        (moved, pairs(&[("q\"\\", "\u{1}\t主")]), Some("up\"st\\re\nam")),
        (
            push(OpPhase::Failed, Some(OpResult::NotMoved { refs: Vec::new() })),
            pairs(&[("a", "1"), ("a", "1")]),
            Some(""),
        ),
    ];
    for (r, values, remote) in cases {
        let line = written(&r, &values, remote);
        assert_eq!(
            one_line(&line).unwrap(),
            found(MarkedRecord {
                record: r,
                values,
                remote: remote.map(str::to_string),
            }),
            "{line}"
        );
    }
}

fn old() -> MarkedRecord {
    MarkedRecord {
        record: record("1", "k1", "land", "a", 10, OpPhase::Intent, None),
        values: Vec::new(),
        remote: None,
    }
}

fn marked_intent() -> MarkedRecord {
    MarkedRecord {
        record: record("2", "k2", "push", "a", 10, OpPhase::Intent, None),
        values: Vec::new(),
        remote: Some("origin".to_string()),
    }
}

fn marked_done() -> MarkedRecord {
    MarkedRecord {
        record: record("2", "k2", "push", "a", 12, OpPhase::Done, Some(OpResult::Moved)),
        values: vec![("n".to_string(), "v".to_string())],
        remote: Some("origin".to_string()),
    }
}

#[test]
fn a_mixed_log_reads_whole_with_marks_where_written() {
    let text = format!("{OLD}\n{MARKED_INTENT}\n{OLD}\n{MARKED_DONE}\n");
    assert_eq!(
        decode_marked_log(text.as_bytes()).unwrap(),
        MarkedLog {
            records: vec![old(), marked_intent(), old(), marked_done()],
            torn_lines: Vec::new(),
        }
    );
}

#[test]
fn a_torn_line_is_kept_by_number() {
    let text = format!("{MARKED_INTENT}\n\n{TORN}\n{OLD}");
    assert_eq!(
        decode_marked_log(text.as_bytes()).unwrap(),
        MarkedLog {
            records: vec![marked_intent(), old()],
            torn_lines: vec![3],
        }
    );
}

#[test]
fn a_malformed_line_is_refused_with_its_number() {
    let bad = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[],"remote":7}"#;
    let text = format!("{OLD}\n{MARKED_INTENT}\n{bad}\n");
    assert_eq!(
        malformed_reason(decode_marked_log(text.as_bytes())),
        (Some(3), "remote is not a string".to_string())
    );
}

/// The record encode_marked_record's lines below carry.
fn push_at_7() -> OpRecord {
    record("o", "k", "push", "a", 7, OpPhase::Intent, None)
}

#[test]
fn no_mark_writes_no_remote_key() {
    assert_eq!(
        written(&push_at_7(), &[], None),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[]}\n"
    );
    assert_eq!(
        written(&push_at_7(), &pairs(&[("n", "v")]), None),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"n\",\"value\":\"v\"}]}\n"
    );
}

#[test]
fn a_mark_without_values_is_the_last_key() {
    assert_eq!(
        written(&push_at_7(), &[], Some("origin")),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"remote\":\"origin\"}\n"
    );
}

#[test]
fn a_mark_goes_after_the_values() {
    assert_eq!(
        written(&push_at_7(), &pairs(&[("n", "v")]), Some("origin")),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"n\",\"value\":\"v\"}],\
         \"remote\":\"origin\"}\n"
    );
}

#[test]
fn a_name_needing_escapes_is_escaped() {
    assert_eq!(
        written(&push_at_7(), &[], Some("up\"st\\re\nam")),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"remote\":\"up\\\"st\\\\re\\nam\"}\n"
    );
}

#[test]
fn a_marked_line_decodes_with_the_mark_dropped() {
    let line = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"done","moves":[],"result":{"outcome":"moved"},"values":[{"name":"t","value":"1"},{"name":"t","value":"é\n"}],"remote":"origin"}"#;
    assert_eq!(
        decode_valued_log(line.as_bytes()).unwrap(),
        ValuedLog {
            records: vec![ValuedRecord {
                record: record("o", "k", "land", "a", 3, OpPhase::Done, Some(OpResult::Moved)),
                values: pairs(&[("t", "1"), ("t", "é\n")]),
            }],
            torn_lines: Vec::new(),
        }
    );
}

#[test]
fn a_mark_goes_after_the_values_on_a_torn_tail() {
    let f = Fixture::new();
    let path = oplog_path(&f.common);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{\"op_id\":\"t\",\"key\":x").unwrap();
    let r = record("o", "k", "land", "simms", 5, OpPhase::Intent, None);
    append_marked_record(&f.common, &r, &pairs(&[("folder_tree", "t1")]), Some("origin")).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "{\"op_id\":\"t\",\"key\":x\
         \n{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":5,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"folder_tree\",\"value\":\"t1\"}],\
         \"remote\":\"origin\"}\n"
    );
}
