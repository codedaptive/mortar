//! status_next: one state per StatusPhase phase gives its step; a stalled move an undo left is finished by
//! `mr undo`, one a rebase left by `mr join` (S-143); a saved folder with no work yet is told to make its
//! change, then save, after the stall, the paused join and unsaved work (S-145);
//! status_phase_from_status_state_tests holds the whole state table.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};

use super::{status_next, StatusState};
use crate::join::{MoveCause, PendingMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(stalled: bool, paused: bool, saved: bool, joined: bool) -> StatusState {
    StatusState {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        paths: StatusPaths::default(),
        saved,
        refused: Vec::new(),
        zone_moved: false,
        joined,
        no_work_yet: false,
        paused: paused.then(|| PausedJoin {
            commit: oid("3333333333333333333333333333333333333333"),
            files: vec![PathBuf::from("src/c.rs")],
        }),
        stalled: stalled.then(|| PendingMove {
            request_ref: "refs/heads/req/core/fix-1".to_string(),
            old: oid("1111111111111111111111111111111111111111"),
            new: oid("2222222222222222222222222222222222222222"),
            op_id: "op-1".to_string(),
            folder_tree: None,
            cause: MoveCause::Rebase,
        }),
        overlaps: Vec::new(),
        gaps: Vec::new(),
    }
}

/// One state per phase as (stalled, paused, saved, joined, the answer for that phase), written out.
const ONE_PER_PHASE: [(bool, bool, bool, bool, &str); 5] = [
    (true, true, false, false, "mr join"),
    (false, true, true, true, "mr resolve"),
    (false, false, false, true, "mr save \"<sentence>\""),
    (false, false, true, false, "mr join"),
    (false, false, true, true, "wait for the zone's tests"),
];

#[test]
fn one_state_per_phase_gives_that_phases_answer() {
    for (stalled, paused, saved, joined, answer) in ONE_PER_PHASE {
        assert_eq!(
            status_next(&state(stalled, paused, saved, joined)),
            answer,
            "stalled {stalled}, paused {paused}, saved {saved}, joined {joined}"
        );
    }
}

/// A stalled state whose move has `cause`, with a paused join and unsaved work that the stall comes before.
fn stalled_by(cause: MoveCause) -> StatusState {
    let mut stalled = state(true, true, false, false);
    stalled.stalled = stalled.stalled.map(|moved| PendingMove { cause, ..moved });
    stalled
}

#[test]
fn a_move_an_undo_left_stalled_is_finished_by_undo() {
    assert_eq!(status_next(&stalled_by(MoveCause::Undo)), "mr undo");
}

#[test]
fn a_move_a_rebase_left_stalled_is_finished_by_join() {
    assert_eq!(status_next(&stalled_by(MoveCause::Rebase)), "mr join");
}

/// `state(stalled, paused, saved, joined)` with no work yet: the branch still at its base, the folder at its tree.
fn no_work(stalled: bool, paused: bool, saved: bool, joined: bool) -> StatusState {
    StatusState { no_work_yet: true, ..state(stalled, paused, saved, joined) }
}

#[test]
fn a_saved_folder_with_no_work_yet_is_told_to_make_its_change_then_save() {
    assert_eq!(status_next(&no_work(false, false, true, false)), "make your change, then mr save");
}

/// S-145's order: the stall, the paused join and unsaved work come before the no-work step.
#[test]
fn a_stall_a_paused_join_and_unsaved_work_come_before_no_work_yet() {
    assert_eq!(status_next(&no_work(true, true, false, false)), "mr join");
    assert_eq!(status_next(&no_work(false, true, true, false)), "mr resolve");
    assert_eq!(status_next(&no_work(false, false, false, false)), "mr save \"<sentence>\"");
}
