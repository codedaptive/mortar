//! log_lines: a head line per record, a line per move (absent old, deleted new), a line for a stale or not-moved
//! result, and one line when nothing matches. Unit tests: oplog's record types and gitio's Oid::parse are plain data.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove};

use super::log_lines;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn record(phase: OpPhase, moves: Vec<RefMove>, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "save-1f".to_string(),
        kind: "save".to_string(),
        actor: "core/fix-1".to_string(),
        reason: "mr save".to_string(),
        at: 1700000000,
        phase,
        moves,
        result,
    }
}

fn moved(refname: &str, old: Option<Oid>, new: Oid) -> RefMove {
    RefMove { refname: refname.to_string(), old, new }
}

const A40: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B40: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[test]
fn nothing_matching_is_one_line() {
    assert_eq!(log_lines(&[]), vec!["no op in the log matches".to_string()]);
}

#[test]
fn a_done_record_is_its_head_then_its_moves() {
    let done = record(
        OpPhase::Done,
        vec![
            moved("refs/heads/req/core/fix-1", Some(oid('a')), oid('b')),
            moved("refs/mortar/snap/core/fix-1", None, oid('b')),
        ],
        Some(OpResult::Moved),
    );
    assert_eq!(
        log_lines(&[&done]),
        vec![
            "op-7 done save by core/fix-1 at 1700000000 key save-1f: mr save".to_string(),
            format!("  refs/heads/req/core/fix-1 {A40} -> {B40}"),
            format!("  refs/mortar/snap/core/fix-1 absent -> {B40}"),
        ]
    );
}

#[test]
fn an_all_zero_new_is_deleted() {
    let intent = record(OpPhase::Intent, vec![moved("refs/heads/req/core/fix-1", Some(oid('a')), oid('0'))], None);
    assert_eq!(
        log_lines(&[&intent]),
        vec![
            "op-7 intent save by core/fix-1 at 1700000000 key save-1f: mr save".to_string(),
            format!("  refs/heads/req/core/fix-1 {A40} -> deleted"),
        ]
    );
}

#[test]
fn a_stale_result_says_what_the_ref_held() {
    let stale = Some(OpResult::Stale { refname: "refs/heads/zone/core".to_string(), actual: Some(oid('b')) });
    let failed = record(OpPhase::Failed, vec![moved("refs/heads/zone/core", Some(oid('a')), oid('b'))], stale);
    assert_eq!(log_lines(&[&failed])[2], format!("  stale: refs/heads/zone/core held {B40}"));
}

#[test]
fn a_not_moved_result_names_each_ref() {
    let refs = vec![("refs/heads/a".to_string(), Some(oid('a'))), ("refs/heads/b".to_string(), None)];
    let failed = record(OpPhase::Failed, Vec::new(), Some(OpResult::NotMoved { refs }));
    assert_eq!(
        log_lines(&[&failed]),
        vec![
            "op-7 failed save by core/fix-1 at 1700000000 key save-1f: mr save".to_string(),
            format!("  not moved: refs/heads/a held {A40}, refs/heads/b was absent"),
        ]
    );
}

#[test]
fn records_follow_one_another_in_order() {
    let mut first = record(OpPhase::Intent, Vec::new(), None);
    first.op_id = "op-1".to_string();
    let second = record(OpPhase::Done, Vec::new(), Some(OpResult::Moved));
    let lines = log_lines(&[&first, &second]);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("op-1 intent "));
    assert!(lines[1].starts_with("op-7 done "));
}
