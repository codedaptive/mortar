use super::{op_lines, OpLine};
use crate::json::parse_json;

/// Two records as oplog writes them (encode_op_record's key order): an intent, and the failed record recover
/// closes it with.
const LOG: &str = r#"{"verb":"log","outcome":"listed","records":[
  {"op_id":"op-1","key":"k","kind":"join-fast-forward","actor":"r1","reason":"join fast-forwards refs/heads/zone/a from aaa to bbb","at":5,"phase":"intent","moves":[{"ref":"refs/heads/zone/a","old":"aaa","new":"bbb"}]},
  {"op_id":"op-1","key":"k","kind":"join-fast-forward","actor":"r1","reason":"join fast-forwards refs/heads/zone/a from aaa to bbb","at":6,"phase":"failed","moves":[{"ref":"refs/heads/zone/a","old":"aaa","new":"bbb"}],"result":{"outcome":"not-moved","refs":[{"ref":"refs/heads/zone/a","actual":"aaa"}]}}
]}"#;

#[test]
fn each_record_is_read_with_its_result_and_moves() {
    let report = parse_json(LOG).expect("the test's JSON parses");
    let intent = OpLine {
        op_id: "op-1".to_string(),
        kind: "join-fast-forward".to_string(),
        phase: "intent".to_string(),
        result: String::new(),
        reason: "join fast-forwards refs/heads/zone/a from aaa to bbb".to_string(),
        moves: vec![("refs/heads/zone/a".to_string(), "bbb".to_string())],
    };
    let failed = OpLine { phase: "failed".to_string(), result: "not-moved".to_string(), ..intent.clone() };
    assert_eq!(op_lines(&report), vec![intent, failed]);
}

#[test]
fn a_record_with_a_marked_remote_and_values_reads_the_same_members() {
    let text = r#"{"records":[{"op_id":"op-9","kind":"publish-push","phase":"done","reason":"r","moves":[],"result":{"outcome":"moved"},"values":[["a","b"]],"remote":"origin"}]}"#;
    let lines = op_lines(&parse_json(text).expect("the test's JSON parses"));
    assert_eq!(lines.len(), 1);
    assert_eq!((lines[0].kind.as_str(), lines[0].result.as_str()), ("publish-push", "moved"));
}

#[test]
fn no_records_member_or_items_that_are_not_objects_give_nothing() {
    assert_eq!(op_lines(&parse_json(r#"{"outcome":"empty"}"#).expect("parses")), Vec::<OpLine>::new());
    assert_eq!(op_lines(&parse_json(r#"{"records":[1,"x",null]}"#).expect("parses")), Vec::<OpLine>::new());
}
