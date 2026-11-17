//! TreeEntryError's Display text: its git error as GitError shows it, and as std::error::Error shows it.
use gitio::{EntryKind, GitError, TreeEntryError};
use std::path::PathBuf;

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
