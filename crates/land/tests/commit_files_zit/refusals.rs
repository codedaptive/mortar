use crate::fixture::{refs, run, Fixture, ACTOR, RECEIPT};
use land::{commit_files, CommitFilesError};
use std::path::PathBuf;

#[test]
fn without_a_landing_nothing_is_written() {
    let fx = Fixture::new();
    let git = fx.add_folder();
    fx.receipt();
    let log = fx.log();
    assert!(matches!(commit_files(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR), Err(CommitFilesError::NotLanding)));
    assert_eq!(fx.branch("integration"), fx.base);
    assert_eq!(fx.log(), log);
    assert_eq!(fx.state(), None);
}

#[test]
fn folder_off_tip_is_refused_without_writes() {
    let (fx, squash, git) = Fixture::landed();
    run(&git, &["checkout", "--detach", fx.base.as_str()]);
    let log = fx.log();
    let state = fx.state();
    assert!(matches!(commit_files(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR),
        Err(CommitFilesError::FolderOffTip { folder: Some(folder), integration }) if folder == fx.base && integration == squash.commit));
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.log(), log);
    assert_eq!(fx.state(), state);
}

#[test]
fn absolute_path_passes_through_snapshot_refusal_without_writes() {
    let (fx, squash, git) = Fixture::landed();
    let log = fx.log();
    let state = fx.state();
    assert!(matches!(commit_files(&git, &refs(), &[PathBuf::from("/outside")], "receipt\n", ACTOR),
        Err(CommitFilesError::Paths(snapshot::CommitPathsError::Absolute { .. }))));
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.log(), log);
    assert_eq!(fx.state(), state);
}
