//! remove: unsaved work, or a file the size cap refused, is NotRemovedUnsaved before any git run.

use std::path::{Path, PathBuf};

use gitio::Git;

use super::remove;
use crate::make::FolderError;

// A git run in this repository would fail with Git, so NotRemovedUnsaved shows nothing ran.
const REPO: &str = "/nonexistent/mortar-wo-folders-5";
const FOLDER: &str = "/nonexistent/mortar-wo-folders-5/folder";

#[test]
fn unsaved_work_is_refused_before_any_git_run() {
    match remove(&Git::new(REPO), Path::new(FOLDER), false, &[], Some(4096), "bilby") {
        Err(FolderError::NotRemovedUnsaved { path, refused, cap }) => {
            assert_eq!(path, PathBuf::from("/nonexistent/mortar-wo-folders-5/folder"));
            assert_eq!(refused, Vec::<PathBuf>::new());
            assert_eq!(cap, Some(4096));
        }
        other => panic!("expected NotRemovedUnsaved, got {other:?}"),
    }
}

#[test]
fn a_refused_file_is_refused_before_any_git_run_even_when_saved() {
    let refused = [PathBuf::from("big.bin")];
    match remove(&Git::new(REPO), Path::new(FOLDER), true, &refused, None, "bilby") {
        Err(FolderError::NotRemovedUnsaved { path, refused, cap }) => {
            assert_eq!(path, PathBuf::from("/nonexistent/mortar-wo-folders-5/folder"));
            assert_eq!(refused, vec![PathBuf::from("big.bin")]);
            assert_eq!(cap, None);
        }
        other => panic!("expected NotRemovedUnsaved, got {other:?}"),
    }
}
