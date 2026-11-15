//! status_next: over the whole state table, the first of stalled, paused, unsaved, not joined decides the step.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};

use super::{status_next, StatusState};
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
    (false, false, false, false, "mortar save \"<sentence>\""),
    (false, false, false, true, "mortar save \"<sentence>\""),
    (false, false, true, false, "mortar join"),
    (false, false, true, true, "wait for the zone's tests"),
    (false, true, false, false, "mortar resolve"),
    (false, true, false, true, "mortar resolve"),
    (false, true, true, false, "mortar resolve"),
    (false, true, true, true, "mortar resolve"),
    (true, false, false, false, "mortar join"),
    (true, false, false, true, "mortar join"),
    (true, false, true, false, "mortar join"),
    (true, false, true, true, "mortar join"),
    (true, true, false, false, "mortar join"),
    (true, true, false, true, "mortar join"),
    (true, true, true, false, "mortar join"),
    (true, true, true, true, "mortar join"),
];

#[test]
fn every_row_of_the_state_table_gives_the_first_that_holds() {
    for (stalled, paused, saved, joined, answer) in TABLE {
        assert_eq!(
            status_next(&state(stalled, paused, saved, joined)),
            answer,
            "stalled {stalled}, paused {paused}, saved {saved}, joined {joined}"
        );
    }
}

#[test]
fn a_stalled_move_comes_before_a_paused_join_and_unsaved_work() {
    assert_eq!(status_next(&state(true, true, false, false)), "mortar join");
}

#[test]
fn a_paused_join_comes_before_unsaved_work() {
    assert_eq!(status_next(&state(false, true, false, true)), "mortar resolve");
}

#[test]
fn unsaved_work_is_saved_with_a_sentence() {
    assert_eq!(status_next(&state(false, false, false, false)), "mortar save \"<sentence>\"");
}

#[test]
fn saved_work_not_joined_is_joined() {
    assert_eq!(status_next(&state(false, false, true, false)), "mortar join");
}

#[test]
fn a_joined_request_waits_for_the_zones_tests() {
    assert_eq!(status_next(&state(false, false, true, true)), "wait for the zone's tests");
}
