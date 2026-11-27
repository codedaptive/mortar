//! pending_base: a rebase's pending move gives the base after that its join op recorded, a joined partner request
//! with none recorded gives its zone move's old tip with no partners, any other op with none recorded gives none; an
//! undo's pending move gives the base before that the reversed op recorded, and none when that op recorded none, is
//! missing or the reason names no op; a value that does not decode is an error naming the op that recorded it.

use folders::{RecordedBase, RequestPartner};
use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove, ValuedRecord};

use super::{pending_base, MoveCause, PendingMove};

const REQUEST: &str = "refs/heads/req/core/fix-1";
const ZONE_REF: &str = "refs/heads/zone/core";
const CUT: &str = "1111111111111111111111111111111111111111";
const ZONE: &str = "2222222222222222222222222222222222222222";
const REBASED: &str = "3333333333333333333333333333333333333333";
const MERGE: &str = "4444444444444444444444444444444444444444";
const WEB: &str = "5555555555555555555555555555555555555555";
const ZONE_FROM: &str = "6666666666666666666666666666666666666666";
const OLD: &str = "7777777777777777777777777777777777777777";

const CUT_TEXT: &str = r#"{"base":"1111111111111111111111111111111111111111","partners":[]}"#;
const ZONE_TEXT: &str = r#"{"base":"2222222222222222222222222222222222222222","partners":[]}"#;
const MERGE_TEXT: &str = r#"{"base":"4444444444444444444444444444444444444444","partners":[{"zone":"web","tip":"5555555555555555555555555555555555555555"}]}"#;
const GARBLED: &str = r#"{"partners":[],"base":"1111111111111111111111111111111111111111"}"#;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn one(refname: &str, old: &str, new: &str) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(oid(old)), new: oid(new) }
}

fn record(op_id: &str, kind: &str, moves: Vec<RefMove>, values: &[(&str, &str)]) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("{kind}:{op_id}"),
            kind: kind.to_string(),
            actor: "agent-a".to_string(),
            reason: "test".to_string(),
            at: 1_700_000_000,
            phase: OpPhase::Done,
            moves,
            result: None,
        },
        values: values.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect(),
    }
}

/// A join-rebase op-5 of the request from OLD to REBASED, with `values`.
fn rebase(values: &[(&str, &str)]) -> ValuedRecord {
    record("op-5", "join-rebase", vec![one(REQUEST, OLD, REBASED)], values)
}

/// A join-zone-request op-5: the zone from ZONE_FROM to REBASED, then the request from OLD to REBASED.
fn joined(values: &[(&str, &str)]) -> ValuedRecord {
    record("op-5", "join-zone-request", vec![one(ZONE_REF, ZONE_FROM, REBASED), one(REQUEST, OLD, REBASED)], values)
}

/// An undo op-undo with `reason`, the request back from REBASED to OLD.
fn undo(reason: &str) -> ValuedRecord {
    let mut valued = record("op-undo", "undo", vec![one(REQUEST, REBASED, OLD)], &[]);
    valued.record.reason = reason.to_string();
    valued
}

fn moved(cause: MoveCause) -> PendingMove {
    let (op_id, old, new) = match cause {
        MoveCause::Rebase => ("op-5", OLD, REBASED),
        MoveCause::Undo => ("op-undo", REBASED, OLD),
    };
    PendingMove {
        request_ref: REQUEST.to_string(),
        old: oid(old),
        new: oid(new),
        op_id: op_id.to_string(),
        folder_tree: None,
        cause,
    }
}

fn plain(hex: &str) -> RecordedBase {
    RecordedBase { base: oid(hex), partners: Vec::new() }
}

/// Each row: the case, the log, the cause of the pending move, and the answer; an Err row holds a word the error
/// must contain.
type Row = (&'static str, Vec<ValuedRecord>, MoveCause, Result<Option<RecordedBase>, &'static str>);

#[test]
fn pending_base_cases() {
    let on_merge =
        RecordedBase { base: oid(MERGE), partners: vec![RequestPartner { zone: "web".to_string(), tip: oid(WEB) }] };
    let recorded = [("base_before", CUT_TEXT), ("base_after", ZONE_TEXT)];
    let of_rebase = "undo of join-rebase op op-5";
    let rows: Vec<Row> = vec![
        ("a plain rebase", vec![rebase(&recorded)], MoveCause::Rebase, Ok(Some(plain(ZONE)))),
        (
            "a partner rebase onto a new merge",
            vec![rebase(&[("base_before", MERGE_TEXT), ("base_after", MERGE_TEXT)])],
            MoveCause::Rebase,
            Ok(Some(on_merge.clone())),
        ),
        ("a rebase that recorded no base", vec![rebase(&[])], MoveCause::Rebase, Ok(None)),
        (
            "a joined partner request",
            vec![joined(&[("base_before", MERGE_TEXT), ("base_after", ZONE_TEXT)])],
            MoveCause::Rebase,
            Ok(Some(plain(ZONE))),
        ),
        ("a joined partner request that recorded no base", vec![joined(&[])], MoveCause::Rebase, Ok(Some(plain(ZONE_FROM)))),
        (
            "a rebase whose base after does not decode",
            vec![rebase(&[("base_before", CUT_TEXT), ("base_after", GARBLED)])],
            MoveCause::Rebase,
            Err("op-5"),
        ),
        ("an undo of a plain rebase", vec![rebase(&recorded), undo(of_rebase)], MoveCause::Undo, Ok(Some(plain(CUT)))),
        (
            "an undo of a partner rebase",
            vec![rebase(&[("base_before", MERGE_TEXT), ("base_after", ZONE_TEXT)]), undo(of_rebase)],
            MoveCause::Undo,
            Ok(Some(on_merge)),
        ),
        ("an undo of a rebase that recorded no base", vec![rebase(&[]), undo(of_rebase)], MoveCause::Undo, Ok(None)),
        ("an undo whose reversed op is missing", vec![undo(of_rebase)], MoveCause::Undo, Ok(None)),
        ("an undo whose reason names no op", vec![rebase(&recorded), undo("put it back")], MoveCause::Undo, Ok(None)),
        (
            "an undo of a rebase whose base before does not decode",
            vec![rebase(&[("base_before", GARBLED), ("base_after", ZONE_TEXT)]), undo(of_rebase)],
            MoveCause::Undo,
            Err("op-5"),
        ),
    ];
    for (case, records, cause, expected) in rows {
        match (pending_base(&records, &moved(cause)), expected) {
            (Err(reason), Err(word)) => assert!(reason.contains(word), "{case}: {reason}"),
            (answer, expected) => assert_eq!(answer, expected.map_err(str::to_string), "{case}"),
        }
    }
}
