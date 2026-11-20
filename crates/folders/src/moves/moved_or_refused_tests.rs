//! moved_or_refused: Moved is Ok; Refused is MoveRefused carrying the folder and the paths in the way.

use std::path::{Path, PathBuf};

use gitio::FolderMoveOutcome;

use super::moved_or_refused::moved_or_refused;
use crate::make::FolderError;

#[test]
fn moved_is_ok() {
    assert!(matches!(moved_or_refused(Path::new("/wt/a"), FolderMoveOutcome::Moved), Ok(())));
}

#[test]
fn refused_names_the_folder_and_the_paths() {
    let outcome = FolderMoveOutcome::Refused(vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")]);
    match moved_or_refused(Path::new("/wt/a"), outcome) {
        Err(FolderError::MoveRefused { path, paths }) => {
            assert_eq!(path, PathBuf::from("/wt/a"));
            assert_eq!(paths, vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")]);
        }
        other => panic!("expected MoveRefused, got {other:?}"),
    }
}
