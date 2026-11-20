//! decide_resolve: each rule in order, the precedence between them, Finish and Rebase carrying the state's saved
//! folder tree (None and Some), and that the markers are read only at the marker check: every step before it gets a
//! marker read that panics if called.

use crate::decide_resolve::decide_resolve;
use crate::join_state::JoinState;
use crate::resolve_step::ResolveStep;
use crate::MarkedFile;
use gitio::Oid;
use std::path::PathBuf;

const REQUEST_TIP: &str = "1111111111111111111111111111111111111111";
const STATE_TREE: &str = "2222222222222222222222222222222222222222";
const CALLER_TREE: &str = "3333333333333333333333333333333333333333";
const REBASED: &str = "4444444444444444444444444444444444444444";
const ELSEWHERE: &str = "5555555555555555555555555555555555555555";
const FOLDER_TREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn state(files_written: bool, rebased_to: Option<&str>) -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid(REQUEST_TIP),
        onto: oid("6666666666666666666666666666666666666666"),
        replayed_tip: oid("7777777777777777777777777777777777777777"),
        commit: oid("8888888888888888888888888888888888888888"),
        tree: oid(STATE_TREE),
        remaining: vec![oid("9999999999999999999999999999999999999999")],
        files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
        all_files: vec![PathBuf::from("old/c.md"), PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
        files_written,
        rebased_to: rebased_to.map(oid),
        folder_tree: None,
    }
}

fn marked(path: &str, lines: &[usize]) -> MarkedFile {
    MarkedFile { path: PathBuf::from(path), lines: lines.to_vec() }
}

/// The marker read for a step that must not read the files.
fn never() -> Result<Vec<MarkedFile>, String> {
    panic!("the conflicted files must not be read for this step")
}

#[test]
fn no_state_is_nothing_to_resolve() {
    let step = decide_resolve(None, Some(&oid(REQUEST_TIP)), never, &oid(CALLER_TREE));
    assert_eq!(step, Ok(ResolveStep::NothingToResolve));
}

#[test]
fn rebased_to_with_the_ref_already_there_finishes() {
    let step = decide_resolve(Some(&state(true, Some(REBASED))), Some(&oid(REBASED)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Finish {
            from: oid("1111111111111111111111111111111111111111"),
            to: oid("4444444444444444444444444444444444444444"),
            folder_tree: None,
        })
    );
}

#[test]
fn ref_moved_elsewhere_is_stale() {
    let step = decide_resolve(Some(&state(true, None)), Some(&oid(ELSEWHERE)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Stale {
            expected: oid("1111111111111111111111111111111111111111"),
            actual: Some(oid("5555555555555555555555555555555555555555")),
        })
    );
}

#[test]
fn ref_gone_is_stale_with_no_actual() {
    let step = decide_resolve(Some(&state(false, None)), None, never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Stale { expected: oid("1111111111111111111111111111111111111111"), actual: None })
    );
}

#[test]
fn rebased_to_with_the_ref_still_at_the_request_tip_rebases_without_reading_markers() {
    let step = decide_resolve(Some(&state(true, Some(REBASED))), Some(&oid(REQUEST_TIP)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Rebase {
            from: oid("1111111111111111111111111111111111111111"),
            to: oid("4444444444444444444444444444444444444444"),
            folder_tree: None,
        })
    );
}

#[test]
fn files_not_written_writes_the_state_tree_and_files_without_reading_markers() {
    let step = decide_resolve(Some(&state(false, None)), Some(&oid(REQUEST_TIP)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::WriteFiles {
            tree: oid("2222222222222222222222222222222222222222"),
            files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
        })
    );
}

#[test]
fn marked_files_are_still_marked_and_only_those_in_order() {
    let read = || Ok::<_, String>(vec![marked("src/a.rs", &[3, 7, 11]), marked("b.txt", &[]), marked("c.md", &[1])]);
    let step = decide_resolve(Some(&state(true, None)), Some(&oid(REQUEST_TIP)), read, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::StillMarked(vec![
            MarkedFile { path: PathBuf::from("src/a.rs"), lines: vec![3, 7, 11] },
            MarkedFile { path: PathBuf::from("c.md"), lines: vec![1] },
        ]))
    );
}

#[test]
fn no_marks_left_continues_with_the_callers_tree() {
    let read = || Ok::<_, String>(vec![marked("src/a.rs", &[]), marked("b.txt", &[])]);
    let step = decide_resolve(Some(&state(true, None)), Some(&oid(REQUEST_TIP)), read, &oid(CALLER_TREE));
    assert_eq!(step, Ok(ResolveStep::Continue { tree: oid("3333333333333333333333333333333333333333") }));
}

#[test]
fn a_marker_read_that_fails_is_returned_as_the_error() {
    let read = || Err::<Vec<MarkedFile>, String>("src/a.rs: denied".to_string());
    let step = decide_resolve(Some(&state(true, None)), Some(&oid(REQUEST_TIP)), read, &oid(CALLER_TREE));
    assert_eq!(step, Err("src/a.rs: denied".to_string()));
}

#[test]
fn ref_moved_is_stale_without_reading_markers() {
    let step = decide_resolve(Some(&state(true, None)), Some(&oid(ELSEWHERE)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Stale {
            expected: oid("1111111111111111111111111111111111111111"),
            actual: Some(oid("5555555555555555555555555555555555555555")),
        })
    );
}

#[test]
fn rebased_to_with_the_ref_elsewhere_is_stale() {
    let step = decide_resolve(Some(&state(true, Some(REBASED))), Some(&oid(ELSEWHERE)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Stale {
            expected: oid("1111111111111111111111111111111111111111"),
            actual: Some(oid("5555555555555555555555555555555555555555")),
        })
    );
}

#[test]
fn finish_carries_the_states_saved_folder_tree() {
    let saved = JoinState { folder_tree: Some(oid(FOLDER_TREE)), ..state(true, Some(REBASED)) };
    let step = decide_resolve(Some(&saved), Some(&oid(REBASED)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Finish {
            from: oid("1111111111111111111111111111111111111111"),
            to: oid("4444444444444444444444444444444444444444"),
            folder_tree: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        })
    );
}

#[test]
fn rebase_carries_the_states_saved_folder_tree() {
    let saved = JoinState { folder_tree: Some(oid(FOLDER_TREE)), ..state(true, Some(REBASED)) };
    let step = decide_resolve(Some(&saved), Some(&oid(REQUEST_TIP)), never, &oid(CALLER_TREE));
    assert_eq!(
        step,
        Ok(ResolveStep::Rebase {
            from: oid("1111111111111111111111111111111111111111"),
            to: oid("4444444444444444444444444444444444444444"),
            folder_tree: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        })
    );
}
