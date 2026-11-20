//! find_undo_target: the last done record moving a ref in scope, by whole name; intent and failed records
//! ignored; an undo's record counts.

use gitio::Oid;

use super::find_undo_target;
use crate::store::{OpPhase, OpRecord, RefMove};

fn rec(op_id: &str, kind: &str, phase: OpPhase, refs: &[&str]) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: format!("key-{op_id}"),
        kind: kind.to_string(),
        actor: "agent-a".to_string(),
        reason: "test".to_string(),
        at: 1763030400,
        phase,
        moves: refs
            .iter()
            .map(|r| RefMove {
                refname: r.to_string(),
                old: None,
                new: Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap(),
            })
            .collect(),
        result: None,
    }
}

fn scope(refs: &[&str]) -> Vec<String> {
    refs.iter().map(|r| r.to_string()).collect()
}

#[test]
fn picks_the_last_of_several() {
    let records = vec![
        rec("op-1", "land", OpPhase::Done, &["refs/heads/main"]),
        rec("op-2", "land", OpPhase::Done, &["refs/heads/other"]),
        rec("op-3", "land", OpPhase::Done, &["refs/heads/landing", "refs/heads/main"]),
        rec("op-4", "land", OpPhase::Done, &["refs/heads/other"]),
    ];
    assert_eq!(find_undo_target(&records, &scope(&["refs/heads/main"])), Some(2));
}

#[test]
fn any_ref_in_scope_matches() {
    let records = vec![
        rec("op-1", "land", OpPhase::Done, &["refs/heads/main"]),
        rec("op-2", "land", OpPhase::Done, &["refs/heads/landing"]),
    ];
    let found = find_undo_target(&records, &scope(&["refs/heads/main", "refs/heads/landing"]));
    assert_eq!(found, Some(1));
}

#[test]
fn ignores_intent_and_failed_records() {
    let records = vec![
        rec("op-1", "land", OpPhase::Done, &["refs/heads/main"]),
        rec("op-2", "land", OpPhase::Intent, &["refs/heads/main"]),
        rec("op-3", "land", OpPhase::Failed, &["refs/heads/main"]),
    ];
    assert_eq!(find_undo_target(&records, &scope(&["refs/heads/main"])), Some(0));
}

#[test]
fn matches_by_whole_name() {
    let records = vec![rec("op-1", "land", OpPhase::Done, &["refs/heads/mainline"])];
    assert_eq!(find_undo_target(&records, &scope(&["refs/heads/main"])), None);
}

#[test]
fn nothing_in_scope_is_none() {
    let records = vec![rec("op-1", "land", OpPhase::Done, &["refs/heads/other"])];
    assert_eq!(find_undo_target(&records, &scope(&["refs/heads/main"])), None);
}

#[test]
fn a_later_undo_is_the_target() {
    let records = vec![
        rec("op-1", "land", OpPhase::Done, &["refs/heads/main"]),
        rec("op-2", "undo", OpPhase::Done, &["refs/heads/main"]),
    ];
    assert_eq!(find_undo_target(&records, &scope(&["refs/heads/main"])), Some(1));
}
