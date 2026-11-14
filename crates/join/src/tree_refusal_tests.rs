//! tree_refusal: a git failure, a killed run included, passes through as Git; each of gitio's refusal shapes,
//! written out by hand, maps to the resolve refusal naming the conflicted path, including a quoted or spaced path and
//! an overlap.

use crate::tree_refusal::tree_refusal;
use crate::JoinRunError;
use gitio::GitError;
use std::path::PathBuf;

fn refusal(stderr: &str) -> GitError {
    GitError { args: Vec::new(), code: None, stderr: stderr.to_string() }
}

fn files() -> Vec<PathBuf> {
    vec![PathBuf::from("src/a.rs"), PathBuf::from("my \"odd\" file.txt"), PathBuf::from("lib"), PathBuf::from("lib/x")]
}

#[test]
fn a_git_run_that_failed_passes_through_unchanged() {
    let error = GitError {
        args: vec!["ls-tree".to_string()],
        code: Some(128),
        stderr: "fatal: not a tree object: \"src/a.rs\" is a folder\n".to_string(),
    };
    let JoinRunError::Git(passed) = tree_refusal(error, &files()) else { panic!("a failed git run must stay Git") };
    assert_eq!(
        passed,
        GitError {
            args: vec!["ls-tree".to_string()],
            code: Some(128),
            stderr: "fatal: not a tree object: \"src/a.rs\" is a folder\n".to_string(),
        }
    );
}

#[test]
fn a_killed_run_with_no_exit_code_passes_through_unchanged() {
    let error = GitError {
        args: vec!["update-index".to_string(), "--index-info".to_string()],
        code: None,
        stderr: "\"src/a.rs\" is a folder".to_string(),
    };
    let JoinRunError::Git(passed) = tree_refusal(error, &files()) else { panic!("a killed git run must stay Git") };
    assert_eq!(
        passed,
        GitError {
            args: vec!["update-index".to_string(), "--index-info".to_string()],
            code: None,
            stderr: "\"src/a.rs\" is a folder".to_string(),
        }
    );
}

#[test]
fn a_path_that_is_a_folder_is_conflicted_path_is_folder() {
    let mapped = tree_refusal(refusal("not a file entry: \"src/a.rs\" is a folder"), &files());
    assert!(
        matches!(&mapped, JoinRunError::ConflictedPathIsFolder { path } if *path == PathBuf::from("src/a.rs")),
        "{mapped:?}"
    );
}

#[test]
fn a_quoted_and_spaced_path_that_is_a_folder_is_named() {
    let mapped = tree_refusal(refusal("not a file entry: \"my \\\"odd\\\" file.txt\" is a folder"), &files());
    let JoinRunError::ConflictedPathIsFolder { path } = mapped else { panic!("{mapped:?}") };
    assert_eq!(path, PathBuf::from("my \"odd\" file.txt"));
}

#[test]
fn an_overlap_names_the_first_conflicted_path_and_is_not_a_folder_refusal() {
    let reason = "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"";
    let mapped = tree_refusal(refusal(reason), &files());
    let JoinRunError::ConflictedPathRefused { path, reason } = mapped else { panic!("{mapped:?}") };
    assert_eq!(path, Some(PathBuf::from("lib")));
    assert_eq!(reason, "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"");
}

#[test]
fn an_overlap_takes_the_first_in_files_order_not_in_the_message() {
    let files = vec![PathBuf::from("lib/x"), PathBuf::from("lib")];
    let reason = "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"";
    let mapped = tree_refusal(refusal(reason), &files);
    let JoinRunError::ConflictedPathRefused { path, .. } = mapped else { panic!("{mapped:?}") };
    assert_eq!(path, Some(PathBuf::from("lib/x")));
}

#[test]
fn a_path_not_as_git_stores_it_is_refused_naming_it() {
    let files = vec![PathBuf::from("ok.txt"), PathBuf::from("a/../b")];
    let reason = "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a/../b\"";
    let JoinRunError::ConflictedPathRefused { path, reason: kept } = tree_refusal(refusal(reason), &files) else {
        panic!("a stored-form refusal must be ConflictedPathRefused");
    };
    assert_eq!(path, Some(PathBuf::from("a/../b")));
    assert_eq!(
        kept,
        "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a/../b\""
    );
}

#[test]
fn a_path_ending_in_a_slash_is_refused_naming_it() {
    let files = vec![PathBuf::from("dir/")];
    let reason = "not a usable path (ends in '/', naming a folder's children): \"dir/\"";
    let mapped = tree_refusal(refusal(reason), &files);
    let JoinRunError::ConflictedPathRefused { path, .. } = mapped else { panic!("{mapped:?}") };
    assert_eq!(path, Some(PathBuf::from("dir/")));
}

#[test]
fn a_refusal_naming_no_conflicted_path_has_no_path() {
    let reason = "not ls-tree records for the paths asked (no final NUL): \"100644 blob\"";
    let JoinRunError::ConflictedPathRefused { path, reason: kept } = tree_refusal(refusal(reason), &files()) else {
        panic!("a refusal naming no path must be ConflictedPathRefused");
    };
    assert_eq!(path, None);
    assert_eq!(kept, "not ls-tree records for the paths asked (no final NUL): \"100644 blob\"");
}

#[test]
fn a_folder_refusal_names_the_path_whose_form_holds_another_in_either_order() {
    let reason = "not a file entry: \"q\\\"x\" is a folder";
    for files in [vec![PathBuf::from("x"), PathBuf::from("q\"x")], vec![PathBuf::from("q\"x"), PathBuf::from("x")]] {
        let mapped = tree_refusal(refusal(reason), &files);
        let JoinRunError::ConflictedPathIsFolder { path } = mapped else { panic!("{mapped:?}") };
        assert_eq!(path, PathBuf::from("q\"x"));
    }
}

#[test]
fn a_single_path_refusal_names_the_path_whose_form_holds_another_in_either_order() {
    let reason = "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"q\\\"x\"";
    for files in [vec![PathBuf::from("x"), PathBuf::from("q\"x")], vec![PathBuf::from("q\"x"), PathBuf::from("x")]] {
        let mapped = tree_refusal(refusal(reason), &files);
        let JoinRunError::ConflictedPathRefused { path, .. } = mapped else { panic!("{mapped:?}") };
        assert_eq!(path, Some(PathBuf::from("q\"x")));
    }
}

#[test]
fn a_path_inside_a_message_of_another_shape_is_not_named() {
    let files = vec![PathBuf::from("x")];
    let reason = "not a file entry: \"x\" is a folder, and more text after it";
    let JoinRunError::ConflictedPathRefused { path, reason: kept } = tree_refusal(refusal(reason), &files) else {
        panic!("a message of no known shape must be ConflictedPathRefused");
    };
    assert_eq!(path, None);
    assert_eq!(kept, "not a file entry: \"x\" is a folder, and more text after it");
}
