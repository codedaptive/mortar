//! encode_valued_record: literal lines written by hand with values (key last, pairs in order, escapes),
//! and with no values the line encode_op_record writes for each phase and result kind.

use gitio::Oid;

use super::{encode_valued_record, OpPhase, OpRecord, OpResult, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(phase: OpPhase, moves: Vec<RefMove>, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "k".to_string(),
        kind: "land".to_string(),
        actor: "simms".to_string(),
        reason: "r".to_string(),
        at: 7,
        phase,
        moves,
        result,
    }
}

fn one_move() -> Vec<RefMove> {
    vec![RefMove {
        refname: "refs/heads/x".to_string(),
        old: None,
        new: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    }]
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

#[test]
fn values_come_after_result_in_the_callers_order() {
    let r = record(OpPhase::Done, Vec::new(), Some(OpResult::Moved));
    assert_eq!(
        encode_valued_record(&r, &pairs(&[("z", "1"), ("a", ""), ("z", "2")])),
        r#"{"op_id":"op-1","key":"k","kind":"land","actor":"simms","reason":"r","at":7,"phase":"done","moves":[],"result":{"outcome":"moved"},"values":[{"name":"z","value":"1"},{"name":"a","value":""},{"name":"z","value":"2"}]}"#
    );
}

#[test]
fn with_no_result_values_follow_moves() {
    let r = record(OpPhase::Intent, one_move(), None);
    assert_eq!(
        encode_valued_record(&r, &pairs(&[("folder_tree", "t")])),
        r#"{"op_id":"op-1","key":"k","kind":"land","actor":"simms","reason":"r","at":7,"phase":"intent","moves":[{"ref":"refs/heads/x","old":null,"new":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}],"values":[{"name":"folder_tree","value":"t"}]}"#
    );
}

#[test]
fn names_and_values_are_escaped() {
    let r = record(OpPhase::Intent, Vec::new(), None);
    assert_eq!(
        encode_valued_record(&r, &pairs(&[("q\"b\\", "l\n\u{1}"), ("", "c\u{1f}\"")])),
        "{\"op_id\":\"op-1\",\"key\":\"k\",\"kind\":\"land\",\"actor\":\"simms\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"q\\\"b\\\\\",\"value\":\"l\\n\\u0001\"},\
         {\"name\":\"\",\"value\":\"c\\u001f\\\"\"}]}"
    );
}

#[test]
fn no_values_writes_no_key_for_an_intent() {
    assert_eq!(
        encode_valued_record(&record(OpPhase::Intent, one_move(), None), &[]),
        r#"{"op_id":"op-1","key":"k","kind":"land","actor":"simms","reason":"r","at":7,"phase":"intent","moves":[{"ref":"refs/heads/x","old":null,"new":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}"#
    );
}

#[test]
fn no_values_writes_no_key_for_each_result_kind() {
    let cases = [
        (
            OpPhase::Done,
            OpResult::Moved,
            r#"{"op_id":"op-1","key":"k","kind":"land","actor":"simms","reason":"r","at":7,"phase":"done","moves":[],"result":{"outcome":"moved"}}"#,
        ),
        (
            OpPhase::Failed,
            OpResult::Stale {
                refname: "refs/heads/x".to_string(),
                actual: None,
            },
            r#"{"op_id":"op-1","key":"k","kind":"land","actor":"simms","reason":"r","at":7,"phase":"failed","moves":[],"result":{"outcome":"stale","ref":"refs/heads/x","actual":null}}"#,
        ),
        (
            OpPhase::Failed,
            OpResult::NotMoved {
                refs: vec![(
                    "refs/heads/x".to_string(),
                    Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
                )],
            },
            r#"{"op_id":"op-1","key":"k","kind":"land","actor":"simms","reason":"r","at":7,"phase":"failed","moves":[],"result":{"outcome":"not-moved","refs":[{"ref":"refs/heads/x","actual":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]}}"#,
        ),
    ];
    for (phase, result, line) in cases {
        assert_eq!(encode_valued_record(&record(phase, Vec::new(), Some(result)), &[]), line);
    }
}
