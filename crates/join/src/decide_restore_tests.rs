//! decide_restore: each rule (Stale, Write with no file, AlreadyRestored, Write over the undone resolve's own
//! finished state, StatePresent), stale winning over every existing case, and other paused joins kept.

use crate::decide_restore::decide_restore;
use crate::join_state::JoinState;
use crate::restore_step::RestoreStep;
use gitio::Oid;
use std::path::PathBuf;

fn oid(digit: char) -> Oid {
    Oid::parse(&digit.to_string().repeat(40)).unwrap()
}

/// The state a resolve's final op recorded: request tip 1, rebased_to and folder_tree null.
fn recorded() -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid('1'),
        onto: oid('2'),
        replayed_tip: oid('3'),
        commit: oid('4'),
        tree: oid('5'),
        remaining: vec![oid('6')],
        files: vec![PathBuf::from("a.txt")],
        all_files: vec![PathBuf::from("a.txt")],
        files_written: true,
        rebased_to: None,
        folder_tree: None,
    }
}

/// The undone resolve's own state as its kill left it: rebased_to and folder_tree saved.
fn finished() -> JoinState {
    JoinState { rebased_to: Some(oid('8')), folder_tree: Some(oid('9')), ..recorded() }
}

fn stale(actual: Option<Oid>) -> RestoreStep {
    RestoreStep::Stale { expected: oid('1'), actual }
}

#[test]
fn a_request_ref_elsewhere_is_stale() {
    assert_eq!(decide_restore(None, &recorded(), Some(&oid('8'))), stale(Some(oid('8'))));
}

#[test]
fn a_request_ref_gone_is_stale_with_no_actual() {
    assert_eq!(decide_restore(None, &recorded(), None), stale(None));
}

#[test]
fn stale_wins_over_every_existing_file() {
    let existing = [
        recorded(),
        finished(),
        JoinState { commit: oid('a'), ..recorded() },
    ];
    for state in &existing {
        assert_eq!(decide_restore(Some(state), &recorded(), Some(&oid('8'))), stale(Some(oid('8'))));
        assert_eq!(decide_restore(Some(state), &recorded(), None), stale(None));
    }
}

#[test]
fn no_file_is_write() {
    assert_eq!(decide_restore(None, &recorded(), Some(&oid('1'))), RestoreStep::Write);
}

#[test]
fn a_file_holding_the_recorded_state_is_already_restored() {
    assert_eq!(decide_restore(Some(&recorded()), &recorded(), Some(&oid('1'))), RestoreStep::AlreadyRestored);
}

#[test]
fn the_undone_resolves_own_finished_state_is_overwritten() {
    assert_eq!(decide_restore(Some(&finished()), &recorded(), Some(&oid('1'))), RestoreStep::Write);
}

#[test]
fn only_rebased_to_or_only_folder_tree_set_is_still_the_same_join() {
    let rebased_only = JoinState { rebased_to: Some(oid('8')), ..recorded() };
    let tree_only = JoinState { folder_tree: Some(oid('9')), ..recorded() };
    assert_eq!(decide_restore(Some(&rebased_only), &recorded(), Some(&oid('1'))), RestoreStep::Write);
    assert_eq!(decide_restore(Some(&tree_only), &recorded(), Some(&oid('1'))), RestoreStep::Write);
}

#[test]
fn another_paused_join_is_state_present() {
    let others = [
        JoinState { commit: oid('a'), ..recorded() },
        JoinState { files_written: false, ..recorded() },
        JoinState { request_ref: "refs/heads/request/r2".to_string(), ..recorded() },
        JoinState { files: vec![PathBuf::from("b.txt")], all_files: vec![PathBuf::from("b.txt")], ..recorded() },
    ];
    for other in &others {
        assert_eq!(decide_restore(Some(other), &recorded(), Some(&oid('1'))), RestoreStep::StatePresent, "{other:?}");
    }
}

#[test]
fn a_finished_state_of_another_join_is_state_present() {
    let other = JoinState { commit: oid('a'), ..finished() };
    assert_eq!(decide_restore(Some(&other), &recorded(), Some(&oid('1'))), RestoreStep::StatePresent);
}
