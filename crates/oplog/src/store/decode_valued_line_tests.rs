//! decode_valued_line: a line with values, one without the key, a bad values member, every cut-short
//! prefix Torn, and encode_valued_record's lines read back.

use gitio::Oid;

use super::{
    decode_valued_line, encode_valued_record, LineError, OpPhase, OpRecord, OpResult, RefMove,
    ValuedRecord,
};

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

#[test]
fn a_line_with_values_gives_the_record_and_its_values() {
    assert_eq!(
        decode_valued_line(WITH_VALUES),
        Ok(ValuedRecord {
            record: record(OpPhase::Done, Some(OpResult::Moved)),
            values: pairs(&[("t", "1"), ("t", "é\n")]),
        })
    );
}

#[test]
fn a_line_without_the_key_has_no_values() {
    let line = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"intent","moves":[]}"#;
    assert_eq!(
        decode_valued_line(line),
        Ok(ValuedRecord {
            record: record(OpPhase::Intent, None),
            values: Vec::new(),
        })
    );
}

#[test]
fn a_bad_values_member_is_malformed() {
    let line = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"intent","moves":[],"values":[{"name":"t"}]}"#;
    assert_eq!(
        decode_valued_line(line),
        Err(LineError::Malformed("value 1: missing key \"value\"".to_string()))
    );
    let twice = r#"{"op_id":"o","key":"k","kind":"land","actor":"a","reason":"r","at":3,"phase":"intent","moves":[],"values":[],"values":[]}"#;
    assert_eq!(
        decode_valued_line(twice),
        Err(LineError::Malformed("the record has key \"values\" twice".to_string()))
    );
}

#[test]
fn every_proper_prefix_is_torn() {
    let mut count = 0;
    for (cut, _) in WITH_VALUES.char_indices() {
        let prefix = &WITH_VALUES[..cut];
        assert_eq!(decode_valued_line(prefix), Err(LineError::Torn), "{prefix:?}");
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
            decode_valued_line(&line),
            Ok(ValuedRecord { record: r, values }),
            "{line}"
        );
    }
}
