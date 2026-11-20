//! join_state_path: the state file's path written out by hand for an absolute, a relative and a linked worktree's git dir.

use crate::join_state_path::join_state_path;
use std::path::{Path, PathBuf};

#[test]
fn absolute_git_dir_gets_mortar_join_json_inside_it() {
    assert_eq!(join_state_path(Path::new("/repo/.git")), PathBuf::from("/repo/.git/mortar-join.json"));
}

#[test]
fn relative_git_dir_stays_relative() {
    assert_eq!(join_state_path(Path::new("a/b")), PathBuf::from("a/b/mortar-join.json"));
}

#[test]
fn linked_worktree_git_dir_gets_its_own_file() {
    assert_eq!(
        join_state_path(Path::new("/repo/.git/worktrees/req-1")),
        PathBuf::from("/repo/.git/worktrees/req-1/mortar-join.json")
    );
}
