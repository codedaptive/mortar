//! decode_marked_log on literal log bytes: a line's mark read back or absent, a mark twice or not a string
//! refused, every proper prefix of a line torn, a mixed log read whole, torn and blank lines skipped and
//! numbered, and each malformed line refused with its number and what is wrong (missing or repeated keys, bad
//! object ids, phases, numbers, fields of the wrong kind, UTF-8 and trailing text). Read through unmarked_log and plain_log, the same
//! bytes give the plain op log. decode_marked_line, the JSON reader and both log mappings are pure.

use gitio::Oid;

use super::{decode_marked_log, unmarked_log, MarkedLog, MarkedRecord};
use crate::store::{plain_log, OpLog, OpPhase, OpRecord, OpResult, OplogError, RefMove};

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

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// `record` as decode_marked_log reads it from a line with no values and no mark.
fn unmarked(record: OpRecord) -> MarkedRecord {
    MarkedRecord { record, values: Vec::new(), remote: None }
}

const STALE_LINE: &str = r#"{"op_id":"op-1","key":"land:zone-a","kind":"land","actor":"agent-a","reason":"squash lands","at":1763030400,"phase":"failed","moves":[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}],"result":{"outcome":"stale","ref":"refs/heads/x","actual":null}}"#;

#[test]
fn decodes_a_literal_line_with_a_stale_result() {
    assert_eq!(
        one_line(STALE_LINE).unwrap(),
        found(unmarked(OpRecord {
            op_id: "op-1".to_string(),
            key: "land:zone-a".to_string(),
            kind: "land".to_string(),
            actor: "agent-a".to_string(),
            reason: "squash lands".to_string(),
            at: 1763030400,
            phase: OpPhase::Failed,
            moves: vec![RefMove {
                refname: "refs/heads/x".to_string(),
                old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
                new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            }],
            result: Some(OpResult::Stale {
                refname: "refs/heads/x".to_string(),
                actual: None,
            }),
        }))
    );
}

#[test]
fn malformed_lines_are_refused_with_what_is_wrong() {
    let cases = [
        (STALE_LINE.replace(r#""kind":"land","#, ""), "missing key \"kind\""),
        (STALE_LINE.replace(r#""at":1763030400,"#, ""), "missing key \"at\""),
        (STALE_LINE.replace(r#","moves":[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]"#, ""), "missing key \"moves\""),
        (STALE_LINE.replacen('{', r#"{"extra":"x","#, 1), "the record has unknown key \"extra\""),
        (STALE_LINE.replacen('{', r#"{"key":"k","#, 1), "the record has key \"key\" twice"),
        (STALE_LINE.replace(r#""new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb""#, r#""new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0""#), "move 1: new \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0\" is not a git object id"),
        (STALE_LINE.replace(r#""old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#, r#""old":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA""#), "move 1: old \"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\" is not a git object id"),
        (STALE_LINE.replace(r#""phase":"failed""#, r#""phase":"Failed""#), "unknown phase \"Failed\" (expected intent, done or failed)"),
        (STALE_LINE.replace(r#""at":1763030400"#, r#""at":"1763030400""#), "at is not a non-negative integer"),
        (STALE_LINE.replace(r#"[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]"#, r#""none""#), "moves is not an array"),
        (STALE_LINE.replace(r#","ref":"refs/heads/x","actual":null}"#, "}"), "missing key \"ref\""),
        (STALE_LINE.replace(r#"{"outcome":"stale","ref":"refs/heads/x","actual":null}"#, "null"), "the result is not a JSON object"),
        ("[]".to_string(), "the record is not a JSON object"),
    ];
    for (line, want) in cases {
        assert_eq!(malformed_reason(one_line(&line)), (Some(1), want.to_string()), "{line}");
    }
}

#[test]
fn bad_numbers_and_trailing_text_are_refused() {
    let cases = [
        (STALE_LINE.replace("1763030400", "-1"), "unexpected '-'"),
        (STALE_LINE.replace("1763030400", "1763030400.5"), "has a fraction or exponent"),
        (STALE_LINE.replace("1763030400", "01763030400"), "has a leading zero"),
        (STALE_LINE.replace("1763030400", "18446744073709551616"), "does not fit 64 bits"),
        (format!("{STALE_LINE} x"), "after the value"),
        (format!("{STALE_LINE}{STALE_LINE}"), "after the value"),
    ];
    for (line, want) in cases {
        let (number, reason) = malformed_reason(one_line(&line));
        assert_eq!(number, Some(1), "{line}");
        assert!(reason.contains(want), "{line}: {reason}");
    }
}

const ESCAPED_LINE: &str = "{\"op_id\":\"op-\\\"7\\\"\",\"key\":\"k\\u0001\",\"kind\":\"land\",\"actor\":\"é主\",\
                    \"reason\":\"\\ud83d\\ude00 \\n\",\"at\":1763030400,\"phase\":\"done\",\
                    \"moves\":[{\"ref\":\"refs/heads/x\",\"old\":null,\"new\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}],\
                    \"result\":{\"outcome\":\"moved\"}}";

#[test]
fn an_escaped_line_decodes_whole() {
    let log = one_line(ESCAPED_LINE).unwrap();
    assert!(log.torn_lines.is_empty());
    let r = &log.records[0].record;
    assert_eq!(r.op_id, "op-\"7\"");
    assert_eq!(r.key, "k\u{1}");
    assert_eq!(r.actor, "é主");
    assert_eq!(r.reason, "😀 \n");
    assert_eq!(r.phase, OpPhase::Done);
}

#[test]
fn every_non_empty_proper_prefix_of_an_escaped_line_is_torn() {
    let mut count = 0;
    for (cut, _) in ESCAPED_LINE.char_indices() {
        let prefix = &ESCAPED_LINE[..cut];
        // decode_marked_log skips an empty line before decoding it, so the empty prefix reads as no line.
        let expected = if prefix.is_empty() {
            MarkedLog::default()
        } else {
            MarkedLog { records: Vec::new(), torn_lines: vec![1] }
        };
        assert_eq!(one_line(prefix).unwrap(), expected, "{prefix:?}");
        count += 1;
    }
    assert_eq!(count, ESCAPED_LINE.chars().count());
}

#[test]
fn complete_json_that_is_not_a_record_is_malformed() {
    assert_eq!(malformed_reason(one_line("{}")), (Some(1), "missing key \"op_id\"".to_string()));
    assert_eq!(malformed_reason(one_line("null")), (Some(1), "the record is not a JSON object".to_string()));
}

#[test]
fn bad_json_is_malformed_not_torn() {
    for line in ["{\"op_id\" \"x\"}", "{\"a\":tru}", "x", "{\"a\":\"\\q\"}"] {
        assert!(matches!(one_line(line), Err(OplogError::Malformed { line: Some(1), .. })), "{line}");
    }
}

#[test]
fn a_boolean_where_a_record_field_belongs_is_malformed_by_the_decoder() {
    let cases = [
        (
            r#"{"op_id":"1","key":"k1","kind":true,"actor":"a","reason":"r","at":10,"phase":"intent","moves":[]}"#,
            "kind is not a string",
        ),
        (
            r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":true,"phase":"intent","moves":[]}"#,
            "at is not a non-negative integer",
        ),
        (
            r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":false}"#,
            "moves is not an array",
        ),
        (
            r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[],"remote":true}"#,
            "remote is not a string",
        ),
        (
            r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[],"values":[{"name":"t","value":true}]}"#,
            "value 1: value is not a string",
        ),
    ];
    for (line, reason) in cases {
        assert_eq!(malformed_reason(one_line(line)), (Some(1), reason.to_string()), "{line}");
    }
}

const L1: &str = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":10,"phase":"intent","moves":[]}"#;
const L2: &str = r#"{"op_id":"1","key":"k1","kind":"land","actor":"a","reason":"r","at":11,"phase":"done","moves":[],"result":{"outcome":"moved"}}"#;
const TORN_L2: &str = r#"{"op_id":"2","key":"k2","ki"#;

fn r1() -> OpRecord {
    record("1", "k1", "land", "a", 10, OpPhase::Intent, None)
}

fn r2() -> OpRecord {
    OpRecord { at: 11, phase: OpPhase::Done, result: Some(OpResult::Moved), ..r1() }
}

/// `bytes` decoded, marks then values dropped: the plain op log.
fn op_log(bytes: &[u8]) -> Result<OpLog, OplogError> {
    decode_marked_log(bytes).map(unmarked_log).map(plain_log)
}

fn plain(records: Vec<OpRecord>, torn_lines: Vec<usize>) -> OpLog {
    OpLog { records, torn_lines }
}

#[test]
fn empty_file_is_an_empty_log() {
    assert_eq!(op_log(b"").unwrap(), plain(Vec::new(), Vec::new()));
}

#[test]
fn records_in_order() {
    let text = format!("{L1}\n{L2}\n");
    assert_eq!(op_log(text.as_bytes()).unwrap(), plain(vec![r1(), r2()], Vec::new()));
}

#[test]
fn last_line_without_newline_that_decodes_is_kept() {
    let text = format!("{L1}\n{L2}");
    assert_eq!(op_log(text.as_bytes()).unwrap(), plain(vec![r1(), r2()], Vec::new()));
}

#[test]
fn torn_tail_is_skipped_and_numbered() {
    let text = format!("{L1}\n{TORN_L2}");
    assert_eq!(op_log(text.as_bytes()).unwrap(), plain(vec![r1()], vec![2]));
}

#[test]
fn torn_middle_line_is_skipped_and_numbered() {
    let text = format!("{TORN_L2}\n{L1}\n{TORN_L2}\n{L2}\n");
    assert_eq!(op_log(text.as_bytes()).unwrap(), plain(vec![r1(), r2()], vec![1, 3]));
}

#[test]
fn blank_lines_are_skipped_but_counted() {
    let text = format!("\n\n{L1}\n\n{TORN_L2}\n\n{L2}\n\n");
    assert_eq!(op_log(text.as_bytes()).unwrap(), plain(vec![r1(), r2()], vec![5]));
}

#[test]
fn utf8_cut_short_at_the_line_end_is_torn() {
    let mut bytes = format!("{L1}\n").into_bytes();
    bytes.extend_from_slice(b"{\"op_id\":\"\xe4\xb8");
    bytes.extend_from_slice(format!("\n{L2}\n").as_bytes());
    assert_eq!(op_log(&bytes).unwrap(), plain(vec![r1(), r2()], vec![2]));
}

#[test]
fn undecodable_middle_line_is_malformed_with_its_number() {
    let text = format!("{L1}\n\n{{\"op_id\":1}}\n{L2}\n");
    assert_eq!(
        malformed_reason(decode_marked_log(text.as_bytes())),
        (Some(3), "op_id is not a string".to_string())
    );
}

#[test]
fn bad_utf8_inside_a_line_is_malformed() {
    let mut bytes = format!("{L1}\n").into_bytes();
    bytes.extend_from_slice(b"{\xff}\n");
    assert_eq!(malformed_reason(decode_marked_log(&bytes)), (Some(2), "not UTF-8 at byte 1".to_string()));
}

#[test]
fn complete_json_with_trailing_text_is_malformed_not_torn() {
    let text = format!("{L1}{L2}\n");
    let (line, reason) = malformed_reason(decode_marked_log(text.as_bytes()));
    assert_eq!(line, Some(1));
    assert!(reason.contains("after the value"), "{reason}");
}
