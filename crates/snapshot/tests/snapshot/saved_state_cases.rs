//! WO-snapshot-10: saved_state read directly against real snapshot refs, and attempt 0 refused before any write.
use crate::support::{run, write, Fixture};
use oplog::oplog_path;
use snapshot::{saved_state, snapshot, RefusedFile, SavedState, SnapshotError};
use std::{fs, path::PathBuf};

const CAP: u64 = 64;

#[test]
fn saved_state_tracks_the_snapshot_ref_and_lists_the_refused_new_file_without_moving_ref_or_log() {
    // Joins saved_state -> snapshot_ref_name -> folder_tree's real capped tree -> read_previous_snapshot of the ref
    // snapshot wrote through oplog apply.
    let f = Fixture::new(&[]);
    let state = || {
        let ref_before = f.git.read_ref(&f.refname).unwrap();
        let log_before = fs::read(oplog_path(&f.common)).ok();
        let state = saved_state(&f.git, &f.repo.path, &f.target, &[], CAP).unwrap();
        assert_eq!(f.git.read_ref(&f.refname).unwrap(), ref_before);
        assert_eq!(fs::read(oplog_path(&f.common)).ok(), log_before);
        state
    };
    assert_eq!(state(), SavedState { saved: false, refused: vec![] });
    snapshot(&f.git, &f.repo.path, &f.target, &[], CAP).unwrap();
    assert_eq!(state(), SavedState { saved: true, refused: vec![] });
    write(&f.repo.path, "over-cap.bin", &[b'x'; 100]);
    let refused = RefusedFile { path: PathBuf::from("over-cap.bin"), size: 100, cap: 64 };
    assert_eq!(state(), SavedState { saved: true, refused: vec![refused.clone()] });
    write(&f.repo.path, "root.txt", b"changed\n");
    assert_eq!(state(), SavedState { saved: false, refused: vec![refused] });
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
