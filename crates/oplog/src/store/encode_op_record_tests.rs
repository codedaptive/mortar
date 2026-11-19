//! encode_op_record: literal lines written by hand for each phase and result, escapes, and no newline.

use gitio::Oid;

use super::{encode_op_record, OpPhase, OpRecord, OpResult, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(phase: OpPhase, moves: Vec<RefMove>, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "land:zone-a".to_string(),
        kind: "land".to_string(),
        actor: "agent-a".to_string(),
        reason: "squash lands".to_string(),
        at: 1763030400,
        phase,
        moves,
        result,
    }
}

fn one_move() -> Vec<RefMove> {
    vec![RefMove {
        refname: "refs/heads/x".to_string(),
        old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    }]
}

#[test]
fn intent_with_create_and_delete_has_no_result() {
    let moves = vec![
        RefMove {
            refname: "refs/heads/new".to_string(),
            old: None,
            new: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        },
        RefMove {
            refname: "refs/heads/old".to_string(),
            old: Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")),
            new: oid("0000000000000000000000000000000000000000"),
        },
    ];
    assert_eq!(
        encode_op_record(&record(OpPhase::Intent, moves, None)),
        r#"{"op_id":"op-1","key":"land:zone-a","kind":"land","actor":"agent-a","reason":"squash lands","at":1763030400,"phase":"intent","moves":[{"ref":"refs/heads/new","old":null,"new":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},{"ref":"refs/heads/old","old":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","new":"0000000000000000000000000000000000000000"}]}"#
    );
}

#[test]
fn done_moved_with_no_moves() {
    let r = OpRecord {
        op_id: "o".to_string(),
        key: "k".to_string(),
        kind: "lock-takeover".to_string(),
        actor: "4242@build-mac".to_string(),
        reason: "r".to_string(),
        at: 0,
        phase: OpPhase::Done,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    };
    assert_eq!(
        encode_op_record(&r),
        r#"{"op_id":"o","key":"k","kind":"lock-takeover","actor":"4242@build-mac","reason":"r","at":0,"phase":"done","moves":[],"result":{"outcome":"moved"}}"#
    );
}

#[test]
fn failed_stale_with_actual_and_absent() {
    let stale = |actual| OpResult::Stale {
        refname: "refs/heads/x".to_string(),
        actual,
    };
    assert_eq!(
        encode_op_record(&record(OpPhase::Failed, one_move(), Some(stale(Some(oid("cccccccccccccccccccccccccccccccccccccccc")))))),
        r#"{"op_id":"op-1","key":"land:zone-a","kind":"land","actor":"agent-a","reason":"squash lands","at":1763030400,"phase":"failed","moves":[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}],"result":{"outcome":"stale","ref":"refs/heads/x","actual":"cccccccccccccccccccccccccccccccccccccccc"}}"#
    );
    assert_eq!(
        encode_op_record(&record(OpPhase::Failed, one_move(), Some(stale(None)))),
        r#"{"op_id":"op-1","key":"land:zone-a","kind":"land","actor":"agent-a","reason":"squash lands","at":1763030400,"phase":"failed","moves":[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}],"result":{"outcome":"stale","ref":"refs/heads/x","actual":null}}"#
    );
}

#[test]
fn failed_not_moved_with_no_refs() {
    assert_eq!(
        encode_op_record(&record(OpPhase::Failed, Vec::new(), Some(OpResult::NotMoved { refs: Vec::new() }))),
        r#"{"op_id":"op-1","key":"land:zone-a","kind":"land","actor":"agent-a","reason":"squash lands","at":1763030400,"phase":"failed","moves":[],"result":{"outcome":"not-moved","refs":[]}}"#
    );
}

#[test]
fn failed_not_moved_with_two_refs_one_absent() {
    let not_moved = OpResult::NotMoved {
        refs: vec![
            ("refs/heads/x".to_string(), Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))),
            ("refs/heads/\"y\"".to_string(), None),
        ],
    };
    assert_eq!(
        encode_op_record(&record(OpPhase::Failed, one_move(), Some(not_moved))),
        r#"{"op_id":"op-1","key":"land:zone-a","kind":"land","actor":"agent-a","reason":"squash lands","at":1763030400,"phase":"failed","moves":[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}],"result":{"outcome":"not-moved","refs":[{"ref":"refs/heads/x","actual":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},{"ref":"refs/heads/\"y\"","actual":null}]}}"#
    );
}

#[test]
fn strings_are_escaped_and_the_line_has_no_newline() {
    let r = OpRecord {
        op_id: "a\"b".to_string(),
        key: "back\\slash".to_string(),
        kind: "k\n".to_string(),
        actor: "tab\there".to_string(),
        reason: "\u{1}é主\u{2028}".to_string(),
        at: u64::MAX,
        phase: OpPhase::Intent,
        moves: vec![RefMove {
            refname: "refs/heads/ü".to_string(),
            old: None,
            new: oid("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"),
        }],
        result: None,
    };
    let line = encode_op_record(&r);
    assert_eq!(
        line,
        "{\"op_id\":\"a\\\"b\",\"key\":\"back\\\\slash\",\"kind\":\"k\\n\",\"actor\":\"tab\\there\",\
         \"reason\":\"\\u0001é主\u{2028}\",\"at\":18446744073709551615,\"phase\":\"intent\",\
         \"moves\":[{\"ref\":\"refs/heads/ü\",\"old\":null,\"new\":\"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\"}]}"
    );
    assert!(!line.contains('\n'));
}
