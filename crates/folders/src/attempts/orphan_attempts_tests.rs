//! orphan_attempts: attempt folders whose branch is not among the refs, as (k, path) in k order; a present branch,
//! a non-canonical tail, another request or zone and a detached worktree not listed.

use std::path::PathBuf;

use gitio::{Oid, Worktree};

use super::orphan_attempts;

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
fn a_folder_whose_branch_is_gone_is_listed_with_its_k() {
    let trees = [worktree("/wt/try-2", Some("refs/heads/try/z/r/2"))];
    assert_eq!(orphan_attempts("z", "r", &[], &trees), vec![(2, PathBuf::from("/wt/try-2"))]);
}

#[test]
fn a_folder_whose_branch_is_present_is_not_listed() {
    let input = refs(&["refs/heads/try/z/r/1"]);
    let trees = [worktree("/wt/try-1", Some("refs/heads/try/z/r/1"))];
    assert_eq!(orphan_attempts("z", "r", &input, &trees), Vec::<(u32, PathBuf)>::new());
}

#[test]
fn a_leading_zero_tail_is_not_listed() {
    let trees = [worktree("/wt/try-01", Some("refs/heads/try/z/r/01"))];
    assert_eq!(orphan_attempts("z", "r", &[], &trees), Vec::<(u32, PathBuf)>::new());
}

#[test]
fn another_request_or_zone_is_not_listed() {
    let trees = [
        worktree("/wt/rx-1", Some("refs/heads/try/z/rx/1")),
        worktree("/wt/y-1", Some("refs/heads/try/y/r/1")),
    ];
    assert_eq!(orphan_attempts("z", "r", &[], &trees), Vec::<(u32, PathBuf)>::new());
}

#[test]
fn a_detached_worktree_is_not_listed() {
    let trees = [worktree("/wt/detached", None)];
    assert_eq!(orphan_attempts("z", "r", &[], &trees), Vec::<(u32, PathBuf)>::new());
}

#[test]
fn orphans_are_listed_in_k_order_whatever_the_input_order() {
    let input = refs(&["refs/heads/try/z/r/2"]);
    let trees = [
        worktree("/wt/try-10", Some("refs/heads/try/z/r/10")),
        worktree("/wt/try-2", Some("refs/heads/try/z/r/2")),
        worktree("/wt/try-3", Some("refs/heads/try/z/r/3")),
        worktree("/wt/try-1", Some("refs/heads/try/z/r/1")),
    ];
    assert_eq!(
        orphan_attempts("z", "r", &input, &trees),
        vec![(1, PathBuf::from("/wt/try-1")), (3, PathBuf::from("/wt/try-3")), (10, PathBuf::from("/wt/try-10"))]
    );
}
