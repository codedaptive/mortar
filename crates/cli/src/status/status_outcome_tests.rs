//! status_outcome: over the whole state table, the first of stalled, paused, unsaved, not joined names the outcome.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};

use super::{status_outcome, StatusState};
use crate::join::PendingMove;

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
        }),
        overlaps: Vec::new(),
        gaps: Vec::new(),
    }
}

/// Every row of the table as (stalled, paused, saved, joined, the answer the spec's order gives), written out.
const TABLE: [(bool, bool, bool, bool, &str); 16] = [
    (false, false, false, false, "unsaved"),
    (false, false, false, true, "unsaved"),
    (false, false, true, false, "saved"),
    (false, false, true, true, "joined"),
    (false, true, false, false, "paused"),
    (false, true, false, true, "paused"),
    (false, true, true, false, "paused"),
    (false, true, true, true, "paused"),
    (true, false, false, false, "stalled"),
    (true, false, false, true, "stalled"),
    (true, false, true, false, "stalled"),
    (true, false, true, true, "stalled"),
    (true, true, false, false, "stalled"),
    (true, true, false, true, "stalled"),
    (true, true, true, false, "stalled"),
    (true, true, true, true, "stalled"),
];

#[test]
fn every_row_of_the_state_table_gives_the_first_that_holds() {
    for (stalled, paused, saved, joined, answer) in TABLE {
        assert_eq!(
            status_outcome(&state(stalled, paused, saved, joined)),
            answer,
            "stalled {stalled}, paused {paused}, saved {saved}, joined {joined}"
        );
    }
}

#[test]
fn a_stalled_move_comes_before_a_paused_join_and_unsaved_work() {
    assert_eq!(status_outcome(&state(true, true, false, false)), "stalled");
}

#[test]
fn a_paused_join_comes_before_unsaved_work() {
    assert_eq!(status_outcome(&state(false, true, false, true)), "paused");
}

#[test]
fn unsaved_saved_and_joined_follow_in_order() {
    assert_eq!(status_outcome(&state(false, false, false, true)), "unsaved");
    assert_eq!(status_outcome(&state(false, false, true, false)), "saved");
    assert_eq!(status_outcome(&state(false, false, true, true)), "joined");
}
