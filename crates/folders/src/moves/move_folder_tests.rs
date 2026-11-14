//! move_folder: unsaved work is refused with NotSaved before any git run, with or without an index tree.

use std::path::{Path, PathBuf};

use gitio::{Git, Oid};

use super::move_folder;
use crate::make::FolderError;

#[test]
fn unsaved_work_is_not_saved_before_any_git_run() {
    // A git run in this repository would fail with Git, so NotSaved shows nothing ran.
    let git = Git::new("/nonexistent/mortar-wo-folders-4");
    let folder = Path::new("/nonexistent/mortar-wo-folders-4/folder");
    let old = Oid::parse("0123456789abcdef0123456789abcdef01234567").unwrap();
    let new = Oid::parse("fedcba9876543210fedcba9876543210fedcba98").unwrap();
    match move_folder(&git, folder, &old, &new, false, "bilby", None) {
        Err(FolderError::NotSaved { path }) => {
            assert_eq!(path, PathBuf::from("/nonexistent/mortar-wo-folders-4/folder"));
        }
        other => panic!("expected NotSaved, got {other:?}"),
    }
}

#[test]
fn unsaved_work_with_an_index_tree_is_not_saved_before_any_git_run() {
    // NotSaved rather than Git shows neither set_index nor the move ran.
    let git = Git::new("/nonexistent/mortar-wo-folders-4b");
    let folder = Path::new("/nonexistent/mortar-wo-folders-4b/folder");
    let old = Oid::parse("0123456789abcdef0123456789abcdef01234567").unwrap();
    let new = Oid::parse("fedcba9876543210fedcba9876543210fedcba98").unwrap();
    let tree = Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    match move_folder(&git, folder, &old, &new, false, "bilby", Some(&tree)) {
        Err(FolderError::NotSaved { path }) => {
            assert_eq!(path, PathBuf::from("/nonexistent/mortar-wo-folders-4b/folder"));
        }
        other => panic!("expected NotSaved, got {other:?}"),
    }
}
