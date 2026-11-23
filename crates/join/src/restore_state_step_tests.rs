//! restore_state_step: no file is Write, the recorded state is AlreadyRestored, the undone resolve's own finished
//! state (both or one of rebased_to and folder_tree set) is Write, and another join's paused or finished state is
//! StatePresent.

use crate::join_state::JoinState;
use crate::restore_state_step::restore_state_step;
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
        base: None,
        base_before: None,
    }
}

/// The undone resolve's own state as its kill left it: rebased_to and folder_tree saved.
fn finished() -> JoinState {
    JoinState { rebased_to: Some(oid('8')), folder_tree: Some(oid('9')), ..recorded() }
}

#[test]
fn no_file_is_write() {
    assert_eq!(restore_state_step(None, &recorded()), RestoreStep::Write);
}

#[test]
fn a_file_holding_the_recorded_state_is_already_restored() {
    assert_eq!(restore_state_step(Some(&recorded()), &recorded()), RestoreStep::AlreadyRestored);
}

#[test]
fn the_undone_resolves_own_finished_state_is_write() {
    assert_eq!(restore_state_step(Some(&finished()), &recorded()), RestoreStep::Write);
}

#[test]
fn only_rebased_to_or_only_folder_tree_set_is_write() {
    let rebased_only = JoinState { rebased_to: Some(oid('8')), ..recorded() };
    let tree_only = JoinState { folder_tree: Some(oid('9')), ..recorded() };
    assert_eq!(restore_state_step(Some(&rebased_only), &recorded()), RestoreStep::Write);
    assert_eq!(restore_state_step(Some(&tree_only), &recorded()), RestoreStep::Write);
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
        assert_eq!(restore_state_step(Some(other), &recorded()), RestoreStep::StatePresent, "{other:?}");
    }
}

#[test]
fn another_joins_finished_state_is_state_present() {
    let other = JoinState { commit: oid('a'), ..finished() };
    assert_eq!(restore_state_step(Some(&other), &recorded()), RestoreStep::StatePresent);
}
