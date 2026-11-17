//! attempt_next: unsaved gives mortar save and saved waits for the harness, whatever else holds; no answer is join
//! or resolve.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};

use super::{attempt_next, StatusState};
use crate::join::{MoveCause, PendingMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(stalled: bool, paused: bool, saved: bool, joined: bool) -> StatusState {
    StatusState {
        zone: "core".to_string(),
        request: "task".to_string(),
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
            request_ref: "refs/heads/try/core/task/1".to_string(),
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

const WAIT: &str = "wait for the harness to keep or drop the attempts";
const SAVE: &str = "mortar save \"<sentence>\"";

/// Every combination of (stalled, paused, saved, joined, the answer): unsaved is always save, saved always wait.
const EVERY_STATE: [(bool, bool, bool, bool, &str); 16] = [
    (false, false, false, false, SAVE),
    (false, false, false, true, SAVE),
    (false, true, false, false, SAVE),
    (false, true, false, true, SAVE),
    (true, false, false, false, SAVE),
    (true, false, false, true, SAVE),
    (true, true, false, false, SAVE),
    (true, true, false, true, SAVE),
    (false, false, true, false, WAIT),
    (false, false, true, true, WAIT),
    (false, true, true, false, WAIT),
    (false, true, true, true, WAIT),
    (true, false, true, false, WAIT),
    (true, false, true, true, WAIT),
    (true, true, true, false, WAIT),
    (true, true, true, true, WAIT),
];

#[test]
fn unsaved_is_save_and_saved_is_wait_whatever_else_holds() {
    for (stalled, paused, saved, joined, answer) in EVERY_STATE {
        assert_eq!(
            attempt_next(&state(stalled, paused, saved, joined)),
            answer,
            "stalled {stalled}, paused {paused}, saved {saved}, joined {joined}"
        );
    }
}

#[test]
fn no_state_answers_join_or_resolve() {
    for (stalled, paused, saved, joined, _) in EVERY_STATE {
        let next = attempt_next(&state(stalled, paused, saved, joined));
        assert!(!next.contains("join") && !next.contains("resolve"), "{next}");
    }
}
