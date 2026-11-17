//! status_lines: one fact per line in the spec's order; empty parts add no line, except the paths, which say there
//! are none. A stalled move's line says whether a rebase or an undo left it (S-143).

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::{Oid, StatusPaths};
use snapshot::RefusedFile;

use super::{status_lines, StatusState};
use crate::join::{MoveCause, PendingMove};

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn bare() -> StatusState {
    StatusState {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        paths: StatusPaths::default(),
        saved: true,
        refused: Vec::new(),
        zone_moved: false,
        joined: false,
        no_work_yet: false,
        paused: None,
        stalled: None,
        overlaps: Vec::new(),
        gaps: Vec::new(),
    }
}

#[test]
fn a_bare_state_names_the_request_says_no_paths_saved_and_zone_not_moved() {
    assert_eq!(
        status_lines(&bare()),
        vec![
            "request fix-1 in zone core",
            "no changed, untracked or conflicted paths",
            "everything is saved",
            "the zone has no commits this request has not taken in",
        ]
    );
}

#[test]
fn each_non_empty_path_list_is_one_line_and_an_empty_one_none() {
    let mut state = bare();
    state.paths = StatusPaths { changed: paths(&["a.rs", "b/c.rs"]), untracked: Vec::new(), conflicted: paths(&["d"]) };
    assert_eq!(status_lines(&state)[1..3].to_vec(), vec!["changed: a.rs, b/c.rs", "conflicted: d"]);
    state.paths = StatusPaths { changed: Vec::new(), untracked: paths(&["new.txt"]), conflicted: Vec::new() };
    assert_eq!(status_lines(&state)[1], "untracked: new.txt");
}

#[test]
fn unsaved_work_is_followed_by_each_refused_file_with_the_cap() {
    let mut state = bare();
    state.saved = false;
    state.refused = vec![
        RefusedFile { path: PathBuf::from("data/big.bin"), size: 2_000_000, cap: 1_048_576 },
        RefusedFile { path: PathBuf::from("run.log"), size: 1_048_577, cap: 1_048_576 },
    ];
    assert_eq!(
        status_lines(&state)[2..5].to_vec(),
        vec![
            "there is work not yet saved",
            "not saved: data/big.bin is 2000000 bytes, over the size cap of 1048576 bytes",
            "not saved: run.log is 1048577 bytes, over the size cap of 1048576 bytes",
        ]
    );
}

#[test]
fn a_moved_zone_and_a_joined_request_are_said_in_that_order() {
    let mut state = bare();
    state.zone_moved = true;
    state.joined = true;
    assert_eq!(
        status_lines(&state)[3..].to_vec(),
        vec!["the zone has commits this request has not taken in", "the request is joined to the zone"]
    );
}

#[test]
fn paused_stalled_overlaps_and_gaps_follow_in_order() {
    let mut state = bare();
    state.paused = Some(PausedJoin {
        commit: oid("3333333333333333333333333333333333333333"),
        files: paths(&["x.rs", "y.rs"]),
    });
    state.stalled = Some(PendingMove {
        request_ref: "refs/heads/req/core/fix-1".to_string(),
        old: oid("1111111111111111111111111111111111111111"),
        new: oid("2222222222222222222222222222222222222222"),
        op_id: "op-1".to_string(),
        folder_tree: None,
        cause: MoveCause::Rebase,
    });
    state.overlaps = vec![("fix-2".to_string(), paths(&["x.rs"])), ("fix-9".to_string(), paths(&["p", "q"]))];
    state.gaps = vec!["docs/a.txt".to_string(), "tools".to_string()];
    assert_eq!(
        status_lines(&state)[4..].to_vec(),
        vec![
            "a join is paused at commit 3333333333333333333333333333333333333333 with conflicted files: x.rs, y.rs",
            "a folder move is stalled: refs/heads/req/core/fix-1 stands at 2222222222222222222222222222222222222222, \
             the folder at 1111111111111111111111111111111111111111",
            "overlaps with fix-2: x.rs",
            "overlaps with fix-9: p, q",
            "gaps (needed outside the work order): docs/a.txt, tools",
        ]
    );
}

#[test]
fn a_move_an_undo_left_stalled_says_the_undo_put_the_branch_back() {
    let mut state = bare();
    state.stalled = Some(PendingMove {
        request_ref: "refs/heads/req/core/fix-1".to_string(),
        old: oid("2222222222222222222222222222222222222222"),
        new: oid("1111111111111111111111111111111111111111"),
        op_id: "op-undo".to_string(),
        folder_tree: None,
        cause: MoveCause::Undo,
    });
    assert_eq!(
        status_lines(&state)[4..].to_vec(),
        vec![
            "an undo's folder move is stalled: the undo put refs/heads/req/core/fix-1 back at \
             1111111111111111111111111111111111111111, the folder at 2222222222222222222222222222222222222222"
        ]
    );
}
