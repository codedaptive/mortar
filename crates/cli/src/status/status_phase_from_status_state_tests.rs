//! From<&StatusState> for StatusPhase: over the whole state table, the first of stalled,
//! paused, unsaved, not joined decides the phase.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};

use super::{StatusPhase, StatusState};
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

/// Every row of the table as (stalled, paused, saved, joined, the phase the precedence order gives), written out.
const TABLE: [(bool, bool, bool, bool, StatusPhase); 16] = [
    (false, false, false, false, StatusPhase::Unsaved),
    (false, false, false, true, StatusPhase::Unsaved),
    (false, false, true, false, StatusPhase::Saved),
    (false, false, true, true, StatusPhase::Joined),
    (false, true, false, false, StatusPhase::Paused),
    (false, true, false, true, StatusPhase::Paused),
    (false, true, true, false, StatusPhase::Paused),
    (false, true, true, true, StatusPhase::Paused),
    (true, false, false, false, StatusPhase::Stalled),
    (true, false, false, true, StatusPhase::Stalled),
    (true, false, true, false, StatusPhase::Stalled),
    (true, false, true, true, StatusPhase::Stalled),
    (true, true, false, false, StatusPhase::Stalled),
    (true, true, false, true, StatusPhase::Stalled),
    (true, true, true, false, StatusPhase::Stalled),
    (true, true, true, true, StatusPhase::Stalled),
];

#[test]
fn every_row_of_the_state_table_gives_the_first_that_holds() {
    for (stalled, paused, saved, joined, phase) in TABLE {
        assert_eq!(
            StatusPhase::from(&state(stalled, paused, saved, joined)),
            phase,
            "stalled {stalled}, paused {paused}, saved {saved}, joined {joined}"
        );
    }
}

#[test]
fn a_stalled_move_comes_before_a_paused_join_and_unsaved_work() {
    assert_eq!(StatusPhase::from(&state(true, true, false, false)), StatusPhase::Stalled);
}

#[test]
fn a_paused_join_comes_before_unsaved_work() {
    assert_eq!(StatusPhase::from(&state(false, true, false, true)), StatusPhase::Paused);
}

#[test]
fn unsaved_saved_and_joined_follow_in_order() {
    assert_eq!(StatusPhase::from(&state(false, false, false, true)), StatusPhase::Unsaved);
    assert_eq!(StatusPhase::from(&state(false, false, true, false)), StatusPhase::Saved);
    assert_eq!(StatusPhase::from(&state(false, false, true, true)), StatusPhase::Joined);
}
