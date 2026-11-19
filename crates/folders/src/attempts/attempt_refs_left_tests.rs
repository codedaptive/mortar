//! attempt_refs_left: RefMoved becomes AttemptRefsLeft with the removed folders; any other error passes unchanged.

use std::path::PathBuf;

use gitio::Oid;

use super::attempt_refs_left::attempt_refs_left;
use crate::make::FolderError;

#[test]
fn ref_moved_becomes_attempt_refs_left_with_the_removed_folders() {
    let err = FolderError::RefMoved {
        refname: "refs/heads/try/folders/r1/2".to_string(),
        at: Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()),
    };
    let removed = [PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")];
    match attempt_refs_left(err, &removed) {
        FolderError::AttemptRefsLeft { moved, at, removed } => {
            assert_eq!(moved, "refs/heads/try/folders/r1/2");
            assert_eq!(at, Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()));
            assert_eq!(removed, vec![PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")]);
        }
        other => panic!("expected AttemptRefsLeft, got {other:?}"),
    }
}

#[test]
fn ref_moved_to_nothing_with_no_folder_removed_keeps_both_empty() {
    let err = FolderError::RefMoved {
        refname: "refs/heads/req/folders/r1".to_string(),
        at: None,
    };
    match attempt_refs_left(err, &[]) {
        FolderError::AttemptRefsLeft { moved, at, removed } => {
            assert_eq!(moved, "refs/heads/req/folders/r1");
            assert_eq!(at, None);
            assert!(removed.is_empty());
        }
        other => panic!("expected AttemptRefsLeft, got {other:?}"),
    }
}

#[test]
fn another_error_passes_through_unchanged() {
    let err = FolderError::AttemptMissing {
        branch: "try/folders/r1/3".to_string(),
    };
    match attempt_refs_left(err, &[PathBuf::from("/wt/try-1")]) {
        FolderError::AttemptMissing { branch } => assert_eq!(branch, "try/folders/r1/3"),
        other => panic!("expected AttemptMissing, got {other:?}"),
    }
}
