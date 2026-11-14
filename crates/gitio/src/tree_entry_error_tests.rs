use crate::{EntryKind, GitError, TreeEntryError};
use std::path::PathBuf;

fn gitlink() -> TreeEntryError {
    TreeEntryError::NotABlob { path: PathBuf::from("sub"), kind: EntryKind::Commit }
}

#[test]
fn not_a_blob_differs_on_path_and_on_kind() {
    assert_ne!(gitlink(), TreeEntryError::NotABlob { path: PathBuf::from("other"), kind: EntryKind::Commit });
    assert_ne!(gitlink(), TreeEntryError::NotABlob { path: PathBuf::from("sub"), kind: EntryKind::Tree });
}

#[test]
fn not_a_blob_is_not_a_git_error_with_its_text() {
    let git = GitError { args: Vec::new(), code: None, stderr: "not a blob: \"sub\" is a \"commit\"".to_string() };
    assert_ne!(gitlink(), TreeEntryError::Git(git));
}

#[test]
fn is_a_std_error_showing_the_not_a_blob_text() {
    let boxed: Box<dyn std::error::Error> = Box::new(gitlink());
    assert_eq!(boxed.to_string(), "not a blob: \"sub\" is a \"commit\"");
}
