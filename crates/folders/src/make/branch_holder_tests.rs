//! branch_holder: the other worktree that has the branch checked out, by full ref.

use std::path::{Path, PathBuf};

use gitio::Worktree;

use super::branch_holder;

fn worktree(path: &str, branch: Option<&str>) -> Worktree {
    Worktree {
        path: PathBuf::from(path),
        head: None,
        branch: branch.map(str::to_string),
        bare: false,
        detached: branch.is_none(),
        locked: false,
        prunable: false,
    }
}

fn list() -> Vec<Worktree> {
    vec![
        worktree("/repo", Some("refs/heads/main")),
        worktree("/wt/detached", None),
        worktree("/wt/folders", Some("refs/heads/zone/folders")),
    ]
}

#[test]
fn names_the_worktree_holding_the_branch() {
    let held = branch_holder(&list(), "refs/heads/zone/folders", Path::new("/wt/new"));
    assert_eq!(held, Some(PathBuf::from("/wt/folders")));
}

#[test]
fn none_when_no_worktree_has_the_branch() {
    assert_eq!(branch_holder(&list(), "refs/heads/zone/join", Path::new("/wt/new")), None);
}

#[test]
fn the_folder_itself_does_not_hold_it() {
    assert_eq!(
        branch_holder(&list(), "refs/heads/zone/folders", Path::new("/wt/folders")),
        None
    );
}

#[test]
fn a_short_name_matches_no_full_ref() {
    assert_eq!(branch_holder(&list(), "zone/folders", Path::new("/wt/new")), None);
}

#[test]
fn an_empty_list_holds_nothing() {
    assert_eq!(branch_holder(&[], "refs/heads/main", Path::new("/wt/new")), None);
}
