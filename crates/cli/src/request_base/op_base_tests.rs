//! op_base: a join-rebase or join-zone-request op's recorded base_before and base_after, as join encodes them, come
//! back as the folders crate's RecordedBase pair, partners in order; an op lacking either value or of another kind is
//! None; a value that does not decode is an error naming its key.

use folders::{RecordedBase, RequestPartner};
use gitio::Oid;
use oplog::{OpPhase, OpRecord, ValuedRecord};

use super::op_base;

const CUT: &str = "1111111111111111111111111111111111111111";
const ZONE: &str = "2222222222222222222222222222222222222222";
const MERGE: &str = "3333333333333333333333333333333333333333";
const WEB: &str = "4444444444444444444444444444444444444444";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn op(kind: &str, values: &[(&str, &str)]) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: "op-1".to_string(),
            key: format!("{kind}:op-1"),
            kind: kind.to_string(),
            actor: "agent-a".to_string(),
            reason: "test".to_string(),
            at: 1_700_000_000,
            phase: OpPhase::Done,
            moves: Vec::new(),
            result: None,
        },
        values: values.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect(),
    }
}

fn plain(hex: &str) -> RecordedBase {
    RecordedBase { base: oid(hex), partners: Vec::new() }
}

/// Each row: the case, the op, and the answer; an Err row holds a word the error must contain.
type Row = (&'static str, ValuedRecord, Result<Option<(RecordedBase, RecordedBase)>, &'static str>);

#[test]
fn op_base_cases() {
    let cut = r#"{"base":"1111111111111111111111111111111111111111","partners":[]}"#;
    let zone = r#"{"base":"2222222222222222222222222222222222222222","partners":[]}"#;
    let merge = r#"{"base":"3333333333333333333333333333333333333333","partners":[{"zone":"web","tip":"4444444444444444444444444444444444444444"}]}"#;
    let on_merge = RecordedBase {
        base: oid(MERGE),
        partners: vec![RequestPartner { zone: "web".to_string(), tip: oid(WEB) }],
    };
    let rows: Vec<Row> = vec![
        (
            "recorded plain",
            op("join-rebase", &[("base_before", cut), ("base_after", zone)]),
            Ok(Some((plain(CUT), plain(ZONE)))),
        ),
        (
            "recorded with a partner",
            op("join-rebase", &[("base_before", cut), ("base_after", merge)]),
            Ok(Some((plain(CUT), on_merge.clone()))),
        ),
        (
            "a joined partner request",
            op("join-zone-request", &[("base_before", merge), ("base_after", zone)]),
            Ok(Some((on_merge, plain(ZONE)))),
        ),
        ("no values", op("join-rebase", &[]), Ok(None)),
        ("base_after missing", op("join-rebase", &[("base_before", cut)]), Ok(None)),
        ("another kind", op("save", &[("base_before", cut), ("base_after", zone)]), Ok(None)),
        (
            "a garbled base_after",
            op("join-rebase", &[("base_before", cut), ("base_after", r#"{"base":"zz","partners":[]}"#)]),
            Err("base_after"),
        ),
    ];
    for (case, valued, expected) in rows {
        match (op_base(&valued), expected) {
            (Err(reason), Err(word)) => assert!(reason.contains(word), "{case}: {reason}"),
            (answer, expected) => assert_eq!(answer, expected.map_err(str::to_string), "{case}"),
        }
    }
}
