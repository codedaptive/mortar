//! classify_conflicted: a blob is written, no entry is absent, gitio's gitlink refusal (written out by hand) is a
//! gitlink, including for a quoted or spaced path; every other error comes back unchanged.

use crate::classify_conflicted::classify_conflicted;
use crate::conflicted_entry::ConflictedEntry;
use gitio::{GitError, Oid};
use std::path::Path;

fn refusal(stderr: &str) -> GitError {
    GitError { args: Vec::new(), code: None, stderr: stderr.to_string() }
}

/// Asserts `error` for `file` comes back as the same error.
fn passes_through(file: &str, error: GitError) {
    assert_eq!(classify_conflicted(Path::new(file), Err(error.clone())), Err(error));
}

#[test]
fn a_blob_is_written() {
    let blob = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    assert_eq!(classify_conflicted(Path::new("src/a.rs"), Ok(Some(blob))), Ok(ConflictedEntry::Write));
}

#[test]
fn no_entry_is_absent() {
    assert_eq!(classify_conflicted(Path::new("old/name.rs"), Ok(None)), Ok(ConflictedEntry::Absent));
}

#[test]
fn the_commit_refusal_naming_the_file_is_a_gitlink() {
    let error = refusal("not a blob: \"vendor/lib\" is a \"commit\"");
    assert_eq!(classify_conflicted(Path::new("vendor/lib"), Err(error)), Ok(ConflictedEntry::Gitlink));
}

#[test]
fn a_path_with_a_space_and_quotes_is_a_gitlink() {
    let error = refusal("not a blob: \"my \\\"odd\\\" sub\" is a \"commit\"");
    assert_eq!(classify_conflicted(Path::new("my \"odd\" sub"), Err(error)), Ok(ConflictedEntry::Gitlink));
}

#[test]
fn a_tree_refusal_passes_through() {
    passes_through("vendor/lib", refusal("not a blob: \"vendor/lib\" is a \"tree\""));
}

#[test]
fn a_commit_refusal_naming_another_path_passes_through() {
    passes_through("vendor/lib", refusal("not a blob: \"vendor/other\" is a \"commit\""));
}

#[test]
fn a_refusal_naming_a_prefix_of_the_file_passes_through() {
    passes_through("a/b", refusal("not a blob: \"a\" is a \"commit\""));
}

#[test]
fn the_commit_text_with_an_exit_code_passes_through() {
    let error = GitError {
        args: Vec::new(),
        code: Some(128),
        stderr: "not a blob: \"vendor/lib\" is a \"commit\"".to_string(),
    };
    passes_through("vendor/lib", error);
}

#[test]
fn the_commit_text_with_args_passes_through() {
    let error = GitError {
        args: vec!["ls-tree".to_string()],
        code: None,
        stderr: "not a blob: \"vendor/lib\" is a \"commit\"".to_string(),
    };
    passes_through("vendor/lib", error);
}
