//! decode_op_record: a literal line, the round trip over many records, and each malformed line refused.

use gitio::Oid;

use super::{decode_op_record, encode_op_record, OpPhase, OpRecord, OpResult, OplogError, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

const LINE: &str = r#"{"op_id":"op-1","key":"land:zone-a","kind":"land","actor":"simms","reason":"squash lands","at":1763030400,"phase":"failed","moves":[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}],"result":{"outcome":"stale","ref":"refs/heads/x","actual":null}}"#;

#[test]
fn decodes_a_literal_line() {
    assert_eq!(
        decode_op_record(LINE).unwrap(),
        OpRecord {
            op_id: "op-1".to_string(),
            key: "land:zone-a".to_string(),
            kind: "land".to_string(),
            actor: "simms".to_string(),
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
        }
    );
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
                    let line = encode_op_record(&r);
                    assert!(!line.contains('\n'), "{line}");
                    assert_eq!(decode_op_record(&line).unwrap(), r, "{line}");
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 324);
}

fn reason_of(line: &str) -> String {
    match decode_op_record(line) {
        Err(OplogError::Malformed { line: None, reason }) => reason,
        other => panic!("{line}: expected Malformed with no line, got {other:?}"),
    }
}

#[test]
fn malformed_lines_are_refused_with_what_is_wrong() {
    let cases = [
        (LINE.replace(r#""kind":"land","#, ""), "missing key \"kind\""),
        (LINE.replace(r#""at":1763030400,"#, ""), "missing key \"at\""),
        (LINE.replace(r#","moves":[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]"#, ""), "missing key \"moves\""),
        (LINE.replacen('{', r#"{"extra":"x","#, 1), "the record has unknown key \"extra\""),
        (LINE.replacen('{', r#"{"key":"k","#, 1), "the record has key \"key\" twice"),
        (LINE.replace(r#""new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb""#, r#""new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0""#), "move 1: new \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0\" is not a git object id"),
        (LINE.replace(r#""old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#, r#""old":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA""#), "move 1: old \"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\" is not a git object id"),
        (LINE.replace(r#""phase":"failed""#, r#""phase":"Failed""#), "unknown phase \"Failed\" (expected intent, done or failed)"),
        (LINE.replace(r#""at":1763030400"#, r#""at":"1763030400""#), "at is not a non-negative integer"),
        (LINE.replace(r#"[{"ref":"refs/heads/x","old":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","new":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]"#, r#""none""#), "moves is not an array"),
        (LINE.replace(r#","ref":"refs/heads/x","actual":null}"#, "}"), "missing key \"ref\""),
        (LINE.replace(r#"{"outcome":"stale","ref":"refs/heads/x","actual":null}"#, "null"), "the result is not a JSON object"),
        ("[]".to_string(), "the record is not a JSON object"),
        (LINE[..40].to_string(), "the record stops before its JSON is complete"),
    ];
    for (line, want) in cases {
        assert_eq!(reason_of(&line), want, "{line}");
    }
}

#[test]
fn bad_numbers_and_trailing_text_are_refused() {
    let cases = [
        (LINE.replace("1763030400", "-1"), "unexpected '-'"),
        (LINE.replace("1763030400", "1763030400.5"), "has a fraction or exponent"),
        (LINE.replace("1763030400", "01763030400"), "has a leading zero"),
        (LINE.replace("1763030400", "18446744073709551616"), "does not fit 64 bits"),
        (format!("{LINE} x"), "after the value"),
        (format!("{LINE}{LINE}"), "after the value"),
    ];
    for (line, want) in cases {
        let reason = reason_of(&line);
        assert!(reason.contains(want), "{line}: {reason}");
    }
}
