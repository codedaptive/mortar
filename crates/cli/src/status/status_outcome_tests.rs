//! status_outcome: one state per StatusPhase phase names its word;
//! status_phase_from_status_state_tests holds the whole state table.

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

/// One state per phase as (stalled, paused, saved, joined, the answer for that phase), written out.
const ONE_PER_PHASE: [(bool, bool, bool, bool, &str); 5] = [
    (true, true, false, false, "stalled"),
    (false, true, true, true, "paused"),
    (false, false, false, true, "unsaved"),
    (false, false, true, false, "saved"),
    (false, false, true, true, "joined"),
];

#[test]
fn one_state_per_phase_gives_that_phases_answer() {
    for (stalled, paused, saved, joined, answer) in ONE_PER_PHASE {
        assert_eq!(
            status_outcome(&state(stalled, paused, saved, joined)),
            answer,
            "stalled {stalled}, paused {paused}, saved {saved}, joined {joined}"
        );
    }
}
