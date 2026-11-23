//! source(): the inner error for Paths and Land; none for NotLanding, NotLanded and FolderOffTip.

use super::{CommitFilesError, LandError};
use gitio::Oid;
use snapshot::CommitPathsError;
use std::error::Error;
use std::path::{Path, PathBuf};

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn paths_source_is_the_inner_commit_paths_error() {
    let error = CommitFilesError::Paths(CommitPathsError::Absolute { path: PathBuf::from("/outside") });
    let path = error.source().and_then(|s| s.downcast_ref::<CommitPathsError>()).map(|e| match e {
        CommitPathsError::Absolute { path } => path.as_path(),
        _ => Path::new("another variant"),
    });
    assert_eq!(path, Some(Path::new("/outside")));
}

#[test]
fn land_source_is_the_inner_land_error() {
    let error = CommitFilesError::Land(LandError::NoZone { commit: oid("1111111111111111111111111111111111111111") });
    let commit = error.source().and_then(|s| s.downcast_ref::<LandError>()).map(|e| match e {
        LandError::NoZone { commit } => commit.to_string(),
        _ => "another variant".to_string(),
    });
    assert_eq!(commit.as_deref(), Some("1111111111111111111111111111111111111111"));
}

#[test]
fn refusals_commit_files_makes_itself_have_no_source() {
    let refusals = [
        CommitFilesError::NotLanding,
        CommitFilesError::NotLanded {
            landed: oid("1111111111111111111111111111111111111111"),
            integration: oid("2222222222222222222222222222222222222222"),
        },
        CommitFilesError::FolderOffTip { folder: None, integration: oid("2222222222222222222222222222222222222222") },
    ];
    for refusal in &refusals {
        assert!(refusal.source().is_none(), "{refusal:?}");
    }
}
