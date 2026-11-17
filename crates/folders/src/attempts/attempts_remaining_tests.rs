//! attempts_remaining: a request's try branches, then its attempt folders whose branch is not among them; refs and
//! branches whose tail is not a canonical attempt number, other requests, zones, req/ branches and detached
//! worktrees not counted.

use std::path::PathBuf;

use gitio::{Oid, Worktree};

use super::attempts_remaining;

const A: &str = "0123456789abcdef0123456789abcdef01234567";

fn refs(names: &[&str]) -> Vec<(Oid, String)> {
    names.iter().map(|name| (Oid::parse(A).unwrap(), name.to_string())).collect()
}

fn worktree(path: &str, branch: Option<&str>) -> Worktree {
    Worktree {
        path: PathBuf::from(path),
        head: Some(Oid::parse(A).unwrap()),
        branch: branch.map(str::to_string),
        bare: false,
        detached: branch.is_none(),
        locked: false,
        prunable: false,
    }
}

#[test]
fn nothing_remains_with_no_refs_and_no_worktrees() {
    assert_eq!(attempts_remaining("z", "r", &[], &[]), Vec::<String>::new());
}

#[test]
fn try_branches_are_named_in_input_order() {
    let input = refs(&["refs/heads/try/z/r/2", "refs/heads/try/z/r/1"]);
    assert_eq!(
        attempts_remaining("z", "r", &input, &[]),
        vec!["refs/heads/try/z/r/2".to_string(), "refs/heads/try/z/r/1".to_string()]
    );
}

#[test]
fn a_folder_whose_branch_is_listed_is_not_named_again() {
    let input = refs(&["refs/heads/try/z/r/1"]);
    let trees = [worktree("/wt/try-1", Some("refs/heads/try/z/r/1"))];
    assert_eq!(attempts_remaining("z", "r", &input, &trees), vec!["refs/heads/try/z/r/1".to_string()]);
}

#[test]
fn folders_whose_branch_is_not_listed_follow_the_branches_in_input_order() {
    let input = refs(&["refs/heads/try/z/r/1"]);
    let trees = [
        worktree("/wt/main", Some("refs/heads/main")),
        worktree("/wt/try-3", Some("refs/heads/try/z/r/3")),
        worktree("/wt/try-1", Some("refs/heads/try/z/r/1")),
        worktree("/wt/try-2", Some("refs/heads/try/z/r/2")),
    ];
    assert_eq!(
        attempts_remaining("z", "r", &input, &trees),
        vec!["refs/heads/try/z/r/1".to_string(), "/wt/try-3".to_string(), "/wt/try-2".to_string()]
    );
}

#[test]
fn a_request_sharing_a_prefix_is_not_counted() {
    let input = refs(&["refs/heads/try/z/rx/1"]);
    let trees = [worktree("/wt/rx-2", Some("refs/heads/try/z/rx/2"))];
    assert_eq!(attempts_remaining("z", "r", &input, &trees), Vec::<String>::new());
}

#[test]
fn another_zone_is_not_counted() {
    let input = refs(&["refs/heads/try/y/r/1"]);
    let trees = [worktree("/wt/y-2", Some("refs/heads/try/y/r/2"))];
    assert_eq!(attempts_remaining("z", "r", &input, &trees), Vec::<String>::new());
}

#[test]
fn request_branches_and_detached_worktrees_are_not_counted() {
    let input = refs(&["refs/heads/req/z/r", "refs/heads/try/z/r"]);
    let trees = [worktree("/wt/req", Some("refs/heads/req/z/r")), worktree("/wt/detached", None)];
    assert_eq!(attempts_remaining("z", "r", &input, &trees), Vec::<String>::new());
}

#[test]
fn a_tail_that_is_not_a_canonical_attempt_number_is_not_counted() {
    let input = refs(&["refs/heads/try/z/r/01", "refs/heads/try/z/r/0", "refs/heads/try/z/r/1/x"]);
    assert_eq!(attempts_remaining("z", "r", &input, &[]), Vec::<String>::new());
}

#[test]
fn a_folder_on_a_branch_with_a_leading_zero_is_not_counted() {
    let trees = [worktree("/wt/try-01", Some("refs/heads/try/z/r/01"))];
    assert_eq!(attempts_remaining("z", "r", &[], &trees), Vec::<String>::new());
}

#[test]
fn only_the_canonical_attempts_among_hand_made_refs_are_counted() {
    let input = refs(&["refs/heads/try/z/r/01", "refs/heads/try/z/r/3", "refs/heads/try/z/r/3/x"]);
    let trees = [
        worktree("/wt/try-0", Some("refs/heads/try/z/r/0")),
        worktree("/wt/try-4", Some("refs/heads/try/z/r/4")),
    ];
    assert_eq!(
        attempts_remaining("z", "r", &input, &trees),
        vec!["refs/heads/try/z/r/3".to_string(), "/wt/try-4".to_string()]
    );
}
