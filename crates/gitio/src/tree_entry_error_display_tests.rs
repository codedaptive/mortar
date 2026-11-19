use crate::{EntryKind, GitError, TreeEntryError};
use std::path::PathBuf;

fn not_a_blob(path: &str, kind: EntryKind) -> String {
    TreeEntryError::NotABlob { path: PathBuf::from(path), kind }.to_string()
}

#[test]
fn a_gitlink_shows_as_a_commit() {
    assert_eq!(not_a_blob("sub", EntryKind::Commit), "not a blob: \"sub\" is a \"commit\"");
}

#[test]
fn a_folder_shows_as_a_tree() {
    assert_eq!(not_a_blob("dir", EntryKind::Tree), "not a blob: \"dir\" is a \"tree\"");
}

#[test]
fn another_type_shows_as_found() {
    assert_eq!(not_a_blob("x", EntryKind::Other("tag".to_string())), "not a blob: \"x\" is a \"tag\"");
}

#[test]
fn the_path_and_type_are_quoted_with_escapes() {
    assert_eq!(
        not_a_blob("n\nl", EntryKind::Other("a\"b".to_string())),
        "not a blob: \"n\\nl\" is a \"a\\\"b\""
    );
}

fn gitlink() -> TreeEntryError {
    TreeEntryError::NotABlob { path: PathBuf::from("sub"), kind: EntryKind::Commit }
}

#[test]
fn git_shows_the_git_errors_text() {
    let error = GitError { args: vec!["ls-tree".to_string()], code: Some(128), stderr: "fatal: bad tree\n".to_string() };
    assert_eq!(TreeEntryError::Git(error).to_string(), "git ls-tree: exit code 128: fatal: bad tree");
}

#[test]
fn is_a_std_error_showing_the_not_a_blob_text() {
    let boxed: Box<dyn std::error::Error> = Box::new(gitlink());
    assert_eq!(boxed.to_string(), "not a blob: \"sub\" is a \"commit\"");
}
