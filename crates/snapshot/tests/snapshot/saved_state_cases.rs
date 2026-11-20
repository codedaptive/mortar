//! saved_state read directly against real snapshot refs, its tree checked against the tree
//! each snapshot ref records, and attempt 0 refused before any write.
use crate::support::{run, tree_names, write, Fixture};
use gitio::Oid;
use oplog::oplog_path;
use snapshot::{saved_state, snapshot, RefusedFile, SavedState, SnapshotError};
use std::{fs, path::PathBuf};

const CAP: u64 = 64;

#[test]
fn saved_state_tracks_the_snapshot_ref_and_lists_the_refused_new_file_without_moving_ref_or_log() {
    // Joins saved_state -> snapshot_ref_name -> folder_tree's real capped tree -> read_previous_snapshot of the ref
    // snapshot wrote through oplog apply. Its tree is checked against the tree the snapshot ref's commit records,
    // read through the test's own git: equal after a snapshot, still equal and without the over-cap file, different
    // after a change, and equal again to the tree a fresh snapshot then records.
    let f = Fixture::new(&[]);
    let state = || {
        let ref_before = f.git.read_ref(&f.refname).unwrap();
        let log_before = fs::read(oplog_path(&f.common)).ok();
        let state = saved_state(&f.git, &f.repo.path, &f.target, &[], CAP).unwrap();
        assert_eq!(f.git.read_ref(&f.refname).unwrap(), ref_before);
        assert_eq!(fs::read(oplog_path(&f.common)).ok(), log_before);
        state
    };
    let ref_tree = || -> Oid { f.git.commit_info(&f.git.read_ref(&f.refname).unwrap().unwrap()).unwrap().tree };
    let unsnapped = state();
    snapshot(&f.git, &f.repo.path, &f.target, &[], CAP).unwrap();
    let first_tree = ref_tree();
    // The folder did not change between the two calls, so the snapshot recorded the tree saved_state built.
    assert_eq!(unsnapped, SavedState { saved: false, tree: first_tree.clone(), refused: vec![] });
    assert_eq!(state(), SavedState { saved: true, tree: first_tree.clone(), refused: vec![] });
    write(&f.repo.path, "over-cap.bin", &[b'x'; 100]);
    let refused = RefusedFile { path: PathBuf::from("over-cap.bin"), size: 100, cap: 64 };
    let capped = state();
    assert_eq!(capped, SavedState { saved: true, tree: first_tree.clone(), refused: vec![refused.clone()] });
    assert!(!tree_names(&f.git, &capped.tree).contains(&"over-cap.bin".to_owned()));
    write(&f.repo.path, "root.txt", b"changed\n");
    let changed = state();
    assert_ne!(changed.tree, first_tree);
    let outcome = snapshot(&f.git, &f.repo.path, &f.target, &[], CAP).unwrap();
    assert_eq!(changed, SavedState { saved: false, tree: ref_tree(), refused: vec![refused] });
    assert_eq!(changed.tree, outcome.tree);
}

#[test]
fn attempt_zero_is_bad_target_for_snapshot_and_saved_state_before_any_ref_log_or_file_write() {
    // Joins snapshot and saved_state -> snapshot_ref_name's refusal -> no oplog apply, no ref under refs/mortar.
    let f = Fixture::new(&[]);
    let mut target = f.target.clone();
    target.attempt = Some(0);
    let log_before = fs::read(oplog_path(&f.common)).ok();
    match snapshot(&f.git, &f.repo.path, &target, &[], CAP) {
        Err(SnapshotError::BadTarget { reason }) => assert_eq!(reason, "attempt 0 is not 1 or more"),
        other => panic!("snapshot: expected BadTarget, got {other:?}"),
    }
    match saved_state(&f.git, &f.repo.path, &target, &[], CAP) {
        Err(SnapshotError::BadTarget { reason }) => assert_eq!(reason, "attempt 0 is not 1 or more"),
        other => panic!("saved_state: expected BadTarget, got {other:?}"),
    }
    assert!(run(&f.git, &["for-each-ref", "refs/mortar"]).is_empty());
    assert_eq!(fs::read(oplog_path(&f.common)).ok(), log_before);
    assert!(run(&f.git, &["status", "--porcelain", "-z"]).is_empty());
    assert_eq!(fs::read(f.repo.path.join("root.txt")).unwrap(), b"base\n");
}
