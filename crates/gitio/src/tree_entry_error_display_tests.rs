use crate::{EntryKind, TreeEntryError};
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
