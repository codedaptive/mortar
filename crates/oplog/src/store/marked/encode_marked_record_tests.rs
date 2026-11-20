//! encode_marked_record: a line with values and a mark has the mark after the values, and every encoded record,
//! with or without values and mark, escapes, every phase and result (stale included) and an `at` of u64::MAX
//! included, is one line that reads back equal through decode_marked_log (pure, like the encoder). The '\n'-ended bytes an append writes are proved through the log file by
//! crates/oplog/tests/oplog/marked_lines.rs.

use gitio::Oid;

use super::{decode_marked_log, encode_marked_record, MarkedLog, MarkedRecord};
use crate::store::{OpPhase, OpRecord, OpResult, RefMove};

fn record(op_id: &str, key: &str, kind: &str, actor: &str, at: u64, phase: OpPhase, result: Option<OpResult>) -> OpRecord {
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

fn push(phase: OpPhase, result: Option<OpResult>) -> OpRecord {
    record("o", "k", "push", "a", 3, phase, result)
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
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
        let line = encode_marked_record(&r, &values, remote);
        assert_eq!(
            decode_marked_log(line.as_bytes()).unwrap(),
            MarkedLog {
                records: vec![MarkedRecord { record: r, values, remote: remote.map(str::to_string) }],
                torn_lines: Vec::new(),
            },
            "{line}"
        );
    }
}

#[test]
fn a_mark_goes_after_the_values() {
    assert_eq!(
        encode_marked_record(
            &record("o", "k", "push", "a", 7, OpPhase::Intent, None),
            &pairs(&[("n", "v")]),
            Some("origin")
        ),
        "{\"op_id\":\"o\",\"key\":\"k\",\"kind\":\"push\",\"actor\":\"a\",\"reason\":\"r\",\"at\":7,\
         \"phase\":\"intent\",\"moves\":[],\"values\":[{\"name\":\"n\",\"value\":\"v\"}],\
         \"remote\":\"origin\"}"
    );
}

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn round_trips_many_records() {
    let phases = [OpPhase::Intent, OpPhase::Done, OpPhase::Failed];
    let results = [
        None,
        Some(OpResult::Moved),
        Some(OpResult::NotMoved { refs: Vec::new() }),
        Some(OpResult::NotMoved {
            refs: vec![
                ("refs/heads/a".to_string(), Some(oid("cccccccccccccccccccccccccccccccccccccccc"))),
                ("refs/heads/\"q\"".to_string(), None),
            ],
        }),
        Some(OpResult::Stale { refname: "refs/heads/x".to_string(), actual: Some(oid("cccccccccccccccccccccccccccccccccccccccc")) }),
        Some(OpResult::Stale { refname: "refs/heads/\"q\"".to_string(), actual: None }),
    ];
    let move_sets = [
        Vec::new(),
        vec![RefMove { refname: "refs/heads/new".to_string(), old: None, new: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") }],
        vec![
            RefMove { refname: "refs/heads/a".to_string(), old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")), new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb") },
            RefMove { refname: "refs/heads/del".to_string(), old: Some(oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")), new: oid("0000000000000000000000000000000000000000") },
            RefMove { refname: "refs/heads/ü".to_string(), old: Some(oid("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd")), new: oid("dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd") },
        ],
    ];
    let control: String = (0u8..0x20).map(char::from).collect();
    let texts = [
        String::new(),
        "land:zone-a".to_string(),
        "say \"hi\" \\ / \n\r\t".to_string(),
        control,
        "é主😀\u{2028}\u{2029}\u{7f}".to_string(),
        "{\"op_id\":1}".to_string(),
    ];
    let ats = [0, 1, 1763030400, u64::MAX];
    let mut count = 0;
    for phase in phases {
        for result in &results {
            for moves in &move_sets {
                for (i, text) in texts.iter().enumerate() {
                    let r = OpRecord {
                        op_id: text.clone(),
                        key: texts[(i + 1) % texts.len()].clone(),
                        kind: texts[(i + 2) % texts.len()].clone(),
                        actor: texts[(i + 3) % texts.len()].clone(),
                        reason: texts[(i + 4) % texts.len()].clone(),
                        at: ats[i % ats.len()],
                        phase,
                        moves: moves.clone(),
                        result: result.clone(),
                    };
                    let line = encode_marked_record(&r, &[], None);
                    assert!(!line.contains('\n'), "{line}");
                    assert_eq!(
                        decode_marked_log(line.as_bytes()).unwrap(),
                        MarkedLog {
                            records: vec![MarkedRecord { record: r, values: Vec::new(), remote: None }],
                            torn_lines: Vec::new(),
                        },
                        "{line}"
                    );
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 324);
}
