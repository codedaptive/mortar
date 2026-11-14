use crate::{EntryKind, GitError, TreeEntryError};
use std::path::PathBuf;

fn from_not_a_blob(path: &str, kind: EntryKind) -> GitError {
    GitError::from(TreeEntryError::NotABlob { path: PathBuf::from(path), kind })
}

#[test]
fn a_gitlink_is_the_refusal_join_matches() {
    let error = from_not_a_blob("sub", EntryKind::Commit);
    assert_eq!(error.stderr, "not a blob: \"sub\" is a \"commit\"");
    assert_eq!(error.code, None);
    assert!(error.args.is_empty());
}

#[test]
fn a_folder_is_a_refusal_naming_a_tree() {
    assert_eq!(
        from_not_a_blob("dir", EntryKind::Tree),
        GitError { args: Vec::new(), code: None, stderr: "not a blob: \"dir\" is a \"tree\"".to_string() }
    );
}

#[test]
fn another_type_is_a_refusal_naming_it_as_found() {
    assert_eq!(
        from_not_a_blob("x", EntryKind::Other("tag".to_string())),
        GitError { args: Vec::new(), code: None, stderr: "not a blob: \"x\" is a \"tag\"".to_string() }
    );
}

#[test]
fn git_comes_back_unchanged() {
    let error = GitError { args: vec!["ls-tree".to_string()], code: Some(128), stderr: "fatal: bad tree".to_string() };
    assert_eq!(
        GitError::from(TreeEntryError::Git(error)),
        GitError { args: vec!["ls-tree".to_string()], code: Some(128), stderr: "fatal: bad tree".to_string() }
    );
}
