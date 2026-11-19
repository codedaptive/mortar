use crate::{GitError, IndexTreeError};
use std::path::PathBuf;

fn unmerged(paths: &[&str]) -> String {
    IndexTreeError::Unmerged { paths: paths.iter().map(PathBuf::from).collect() }.to_string()
}

#[test]
fn one_path_is_named_quoted() {
    assert_eq!(unmerged(&["top.txt"]), "index has unmerged paths: \"top.txt\"");
}

#[test]
fn several_paths_are_named_in_order_joined_by_commas() {
    assert_eq!(
        unmerged(&["sp ace", "sub/inner.txt", "top.txt"]),
        "index has unmerged paths: \"sp ace\", \"sub/inner.txt\", \"top.txt\""
    );
}

#[test]
fn a_newline_and_a_quote_in_a_path_show_escaped() {
    assert_eq!(unmerged(&["nl\nname", "a\"b"]), "index has unmerged paths: \"nl\\nname\", \"a\\\"b\"");
}

#[test]
fn git_shows_the_git_errors_text() {
    let error = GitError { args: vec!["write-tree".to_string()], code: Some(128), stderr: "fatal: oops\n".to_string() };
    assert_eq!(IndexTreeError::Git(error).to_string(), "git write-tree: exit code 128: fatal: oops");
}

#[test]
fn a_boxed_std_error_shows_the_unmerged_text() {
    let paths = ["a", "sp ace"].iter().map(PathBuf::from).collect();
    let boxed: Box<dyn std::error::Error> = Box::new(IndexTreeError::Unmerged { paths });
    assert_eq!(boxed.to_string(), "index has unmerged paths: \"a\", \"sp ace\"");
}
