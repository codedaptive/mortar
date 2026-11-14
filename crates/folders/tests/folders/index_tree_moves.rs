//! WO-folders-4c, S-87/S-88: move_folder with an index tree t, the four index-tree cases named in
//! move_folder.rs's header, each on its own throwaway repository. Joins folders::move_folder -> gitio
//! set_index/folder_move -> real sparse index and files, under the caller's lock with oplog::recover first.
use super::index_tree_support::{TreeFixture, ADDED, KEEP, RESOLVED, ZONE};
use crate::support::write;
use folders::FolderError;
use oplog::HeldLockError;
use std::path::PathBuf;

const LOCAL_EDIT: &[u8] = b"local edit outside the zone and the resolve\n";

/// The folder after a move to N: N's zone bytes, the resolution, N's added path, and the local edit on KEEP kept
/// and shown as modified.
fn assert_files_at_new(m: &TreeFixture) {
    assert_eq!(m.read(ZONE), b"zone change at new\n");
    assert_eq!(m.read(RESOLVED), b"resolution bytes\n");
    assert_eq!(m.read(ADDED), b"added at new\n");
    assert_eq!(m.read(KEEP), LOCAL_EDIT);
    assert_eq!(m.status(KEEP), b" M kit/zone/keep\n");
}

#[test]
fn after_a_resolve_index_and_files_reach_new_with_other_edits_kept_and_no_op_logged() {
    let m = TreeFixture::new();
    write(&m.folder, KEEP, LOCAL_EDIT);
    m.branch_at_new();
    let log = m.log();
    m.move_with_tree().unwrap();
    assert_files_at_new(&m);
    assert_eq!(m.log(), log, "the branch was already at N: no op");
    assert_eq!(m.branch(), Some(m.new.clone()));
    // The index reaches N's tree.
    assert_eq!(m.index_tree(), m.tree_of(&m.new));
}

#[test]
fn a_repeat_after_a_completed_move_finishes_with_no_refusal_and_changes_nothing() {
    let m = TreeFixture::new();
    write(&m.folder, KEEP, LOCAL_EDIT);
    m.branch_at_new();
    let log = m.log();
    m.move_with_tree().unwrap();
    let first = (m.index_tree(), m.read(ZONE), m.read(RESOLVED), m.read(ADDED), m.read(KEEP), m.log());
    m.move_with_tree().unwrap();
    assert_eq!((m.index_tree(), m.read(ZONE), m.read(RESOLVED), m.read(ADDED), m.read(KEEP), m.log()), first);
    assert_files_at_new(&m);
    assert_eq!(m.log(), log);
    assert_eq!(m.branch(), Some(m.new.clone()));
    // The repeat leaves the index at N's tree, not at t.
    assert_eq!(m.index_tree(), m.tree_of(&m.new));
}

#[test]
fn killed_between_set_index_and_the_move_finishes_once_on_repeat() {
    let m = TreeFixture::new();
    write(&m.folder, KEEP, LOCAL_EDIT);
    m.branch_at_new();
    // Simulates the kill point: the earlier run set the folder's index to t and died before the move.
    m.f.git.set_index(&m.folder, &m.tree).unwrap();
    assert_eq!(m.index_tree(), format!("{}\n", m.tree).into_bytes());
    let log = m.log();
    m.move_with_tree().unwrap();
    assert_eq!(m.index_tree(), m.tree_of(&m.new));
    assert_eq!(m.read(RESOLVED), b"resolution bytes\n");
    assert_eq!(m.read(ADDED), b"added at new\n");
    assert_eq!(m.read(KEEP), LOCAL_EDIT);
    assert_eq!(m.status(KEEP), b" M kit/zone/keep\n");
    assert_eq!(m.log(), log);
    // The zone path has N's bytes.
    assert_eq!(m.read(ZONE), b"zone change at new\n");
}

#[test]
fn a_local_edit_on_a_zone_changed_path_is_refused_with_the_index_at_t_and_the_branch_unmoved() {
    let m = TreeFixture::new();
    let edit = b"local edit on the zone path\n";
    write(&m.folder, ZONE, edit);
    let log = m.log();
    let error = m.move_with_tree().expect_err("an edit on a path the zone changed must refuse");
    let HeldLockError::Work(FolderError::MoveRefused { path, paths }) = error else {
        panic!("expected MoveRefused: {error:?}")
    };
    assert_eq!(path, m.folder);
    assert_eq!(paths, vec![PathBuf::from(ZONE)]);
    assert_eq!(m.index_tree(), format!("{}\n", m.tree).into_bytes());
    assert_eq!(m.read(ZONE), edit);
    assert_eq!(m.log(), log);
    // The branch is not moved from O.
    assert_eq!(m.branch(), Some(m.old.clone()));
}
