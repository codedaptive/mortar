//! find_pending_move: a done join-rebase that is the request ref's last mover, with the branch at its new tip, is a
//! pending folder move carrying the folder_tree that op recorded, if any (cause Rebase); a done undo of a join-rebase
//! op, last mover with the branch at its new tip, is one too, from the rebased tip back (cause Undo, no tree, S-143).
//! The undo cases a resolve step, a missing or only-failed reversed op decide are ZITs in
//! crates/cli/tests/find_pending_move_resolve_tests.rs.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove, ValuedRecord};

use super::{find_pending_move, MoveCause, PendingMove};

const REQUEST: &str = "refs/heads/req/core/fix-1";
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const LATER: &str = "3333333333333333333333333333333333333333";
const TREE: &str = "4444444444444444444444444444444444444444";
const OTHER_TREE: &str = "5555555555555555555555555555555555555555";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(op_id: &str, kind: &str, phase: OpPhase, refname: &str, old: Option<&str>, new: &str) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("{kind}:{op_id}"),
            kind: kind.to_string(),
            actor: "agent-a".to_string(),
            reason: "test".to_string(),
            at: 1_700_000_000,
            phase,
            moves: vec![RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }],
            result: None,
        },
        values: Vec::new(),
    }
}

fn with_values(mut valued: ValuedRecord, values: &[(&str, &str)]) -> ValuedRecord {
    valued.values = values.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect();
    valued
}

fn rebase() -> ValuedRecord {
    record("op-rebase", "join-rebase", OpPhase::Done, REQUEST, Some(OLD), NEW)
}

fn pending() -> PendingMove {
    PendingMove {
        request_ref: REQUEST.to_string(),
        old: oid(OLD),
        new: oid(NEW),
        op_id: "op-rebase".to_string(),
        folder_tree: None,
        cause: MoveCause::Rebase,
    }
}

fn pending_from(tree: &str) -> PendingMove {
    PendingMove { folder_tree: Some(oid(tree)), ..pending() }
}

#[test]
fn an_empty_log_has_none() {
    assert_eq!(find_pending_move(&[], REQUEST, Some(&oid(NEW))), None);
}

#[test]
fn a_save_as_last_mover_is_none() {
    let records = [record("op-save", "save", OpPhase::Done, REQUEST, Some(OLD), NEW)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), None);
}

#[test]
fn a_done_join_rebase_with_the_branch_at_its_new_tip_is_the_move() {
    assert_eq!(find_pending_move(&[rebase()], REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_branch_elsewhere_is_none() {
    assert_eq!(find_pending_move(&[rebase()], REQUEST, Some(&oid(LATER))), None);
    assert_eq!(find_pending_move(&[rebase()], REQUEST, Some(&oid(OLD))), None);
}

#[test]
fn an_absent_branch_is_none() {
    assert_eq!(find_pending_move(&[rebase()], REQUEST, None), None);
}

#[test]
fn a_later_save_of_the_same_ref_ends_it() {
    let records = [rebase(), record("op-save", "save", OpPhase::Done, REQUEST, Some(NEW), LATER)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(LATER))), None);
}

#[test]
fn later_failed_and_intent_records_are_ignored() {
    let records = [
        rebase(),
        record("op-failed", "save", OpPhase::Failed, REQUEST, Some(NEW), LATER),
        record("op-intent", "save", OpPhase::Intent, REQUEST, Some(NEW), LATER),
    ];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_later_record_of_another_ref_is_ignored() {
    let records = [rebase(), record("op-zone", "join-ff", OpPhase::Done, "refs/heads/zone/core", Some(NEW), LATER)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_later_move_that_changes_nothing_is_ignored() {
    let records = [rebase(), record("op-same", "save", OpPhase::Done, REQUEST, Some(NEW), NEW)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_join_rebase_without_an_old_tip_is_none() {
    let records = [record("op-rebase", "join-rebase", OpPhase::Done, REQUEST, None, NEW)];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), None);
}

#[test]
fn a_join_rebase_carrying_folder_tree_gives_that_tree() {
    let records = [with_values(rebase(), &[("folder_tree", TREE)])];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending_from(TREE)));
}

#[test]
fn a_join_rebase_with_no_values_gives_no_tree() {
    let records = [with_values(rebase(), &[])];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn a_folder_tree_that_does_not_parse_gives_no_tree() {
    let records = [with_values(rebase(), &[("folder_tree", "not-a-tree")])];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending()));
}

#[test]
fn the_first_of_two_folder_tree_pairs_is_used() {
    let records = [with_values(rebase(), &[("other", OTHER_TREE), ("folder_tree", TREE), ("folder_tree", OTHER_TREE)])];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending_from(TREE)));
}

#[test]
fn a_later_record_of_another_ref_carrying_folder_tree_is_not_read() {
    let zone = record("op-zone", "join-rebase", OpPhase::Done, "refs/heads/zone/core", Some(NEW), LATER);
    let records = [with_values(rebase(), &[("folder_tree", TREE)]), with_values(zone, &[("folder_tree", OTHER_TREE)])];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(NEW))), Some(pending_from(TREE)));
}

#[test]
fn a_later_save_ends_a_move_that_carried_a_tree() {
    let records = [
        with_values(rebase(), &[("folder_tree", TREE)]),
        record("op-save", "save", OpPhase::Done, REQUEST, Some(NEW), LATER),
    ];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(LATER))), None);
}

const UNDO_OF_REBASE: &str = "undo of join-rebase op op-5";

/// The join-rebase op-5 moved the request from OLD to the rebased tip NEW.
fn rebase_5() -> ValuedRecord {
    record("op-5", "join-rebase", OpPhase::Done, REQUEST, Some(OLD), NEW)
}

/// An undo op whose move of the request ref goes from `old` back to OLD, with `reason`.
fn undo(reason: &str, old: Option<&str>) -> ValuedRecord {
    let mut valued = record("op-undo", "undo", OpPhase::Done, REQUEST, old, OLD);
    valued.record.reason = reason.to_string();
    valued
}

fn undone() -> PendingMove {
    PendingMove {
        request_ref: REQUEST.to_string(),
        old: oid(NEW),
        new: oid(OLD),
        op_id: "op-undo".to_string(),
        folder_tree: None,
        cause: MoveCause::Undo,
    }
}

#[test]
fn an_undo_of_a_join_rebase_with_the_branch_back_at_the_old_tip_is_the_move_back() {
    let records = [rebase_5(), undo(UNDO_OF_REBASE, Some(NEW))];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(OLD))), Some(undone()));
}

#[test]
fn an_undos_move_back_carries_no_tree_even_when_its_record_has_one() {
    let records = [rebase_5(), with_values(undo(UNDO_OF_REBASE, Some(NEW)), &[("folder_tree", TREE)])];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(OLD))), Some(undone()));
}

#[test]
fn an_undo_that_leaves_no_move_back_pending_is_none() {
    let save_5 = record("op-5", "save", OpPhase::Done, REQUEST, Some(OLD), NEW);
    let save_after = record("op-save", "save", OpPhase::Done, REQUEST, Some(OLD), LATER);
    let cases: [(&str, Vec<ValuedRecord>, &str); 7] = [
        ("the branch elsewhere", vec![rebase_5(), undo(UNDO_OF_REBASE, Some(NEW))], LATER),
        ("the branch at the rebased tip", vec![rebase_5(), undo(UNDO_OF_REBASE, Some(NEW))], NEW),
        ("an undo of a save", vec![save_5, undo("undo of save op op-5", Some(NEW))], OLD),
        ("an unreadable reason", vec![rebase_5(), undo("put the request back", Some(NEW))], OLD),
        ("an empty reason", vec![rebase_5(), undo("", Some(NEW))], OLD),
        ("a move with no old", vec![rebase_5(), undo(UNDO_OF_REBASE, None)], OLD),
        ("a later save", vec![rebase_5(), undo(UNDO_OF_REBASE, Some(NEW)), save_after], LATER),
    ];
    for (case, records, branch_at) in cases {
        assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(branch_at))), None, "{case}");
    }
}
