//! tree_refusal: a folder refusal is ConflictedPathIsFolder naming its path as given; a Git failure with an exit
//! code or args passes through equal; a wrong-shape refusal (no code, no args) is refused naming no path.
//! The typed refusals whose reason is gitio's Display are tested in tests/join_rit/tree_refusals.rs,
//! except an overlap with neither path in files, which resolve cannot reach.

use crate::tree_refusal::tree_refusal;
use crate::JoinRunError;
use gitio::{GitError, TreeWithPathsError, TreeWithPathsRefusal};
use std::path::PathBuf;

fn p(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn files() -> Vec<PathBuf> {
    vec![p("src/a.rs"), p("my \"odd\" file.txt"), p("lib"), p("lib/x")]
}

fn refused(refusal: TreeWithPathsRefusal, files: &[PathBuf]) -> JoinRunError {
    tree_refusal(TreeWithPathsError::Refused(refusal), files)
}

fn is_folder(mapped: JoinRunError) -> PathBuf {
    let JoinRunError::ConflictedPathIsFolder { path } = mapped else { panic!("want ConflictedPathIsFolder: {mapped:?}") };
    path
}

fn path_refused(mapped: JoinRunError) -> (Option<PathBuf>, String) {
    let JoinRunError::ConflictedPathRefused { path, reason } = mapped else {
        panic!("want ConflictedPathRefused: {mapped:?}")
    };
    (path, reason)
}

#[test]
fn a_folder_in_from_is_conflicted_path_is_folder_naming_it() {
    assert_eq!(is_folder(refused(TreeWithPathsRefusal::FolderInFrom { path: p("src/a.rs") }, &files())), p("src/a.rs"));
}

#[test]
fn a_folder_in_base_is_conflicted_path_is_folder_naming_it() {
    assert_eq!(is_folder(refused(TreeWithPathsRefusal::FolderInBase { path: p("lib") }, &files())), p("lib"));
}

#[test]
fn a_quoted_and_spaced_path_is_carried_through_unchanged() {
    let path = p("my \"odd\" file.txt");
    assert_eq!(is_folder(refused(TreeWithPathsRefusal::FolderInFrom { path }, &files())), p("my \"odd\" file.txt"));
}

#[test]
fn a_git_run_with_an_exit_code_passes_through_equal() {
    let error = GitError { args: vec!["ls-tree".to_string()], code: Some(128), stderr: "fatal: bad tree\n".to_string() };
    let JoinRunError::Git(passed) = tree_refusal(TreeWithPathsError::Git(error), &files()) else {
        panic!("a failed git run must stay Git")
    };
    assert_eq!(
        passed,
        GitError { args: vec!["ls-tree".to_string()], code: Some(128), stderr: "fatal: bad tree\n".to_string() }
    );
}

#[test]
fn a_killed_run_with_args_and_no_exit_code_passes_through_equal() {
    let error = GitError { args: vec!["update-index".to_string()], code: None, stderr: "killed".to_string() };
    let JoinRunError::Git(passed) = tree_refusal(TreeWithPathsError::Git(error), &files()) else {
        panic!("a killed git run must stay Git")
    };
    assert_eq!(passed, GitError { args: vec!["update-index".to_string()], code: None, stderr: "killed".to_string() });
}

#[test]
fn a_wrong_shape_refusal_with_no_code_and_no_args_is_refused_naming_no_path() {
    let stderr = "not ls-tree records for the paths asked (no final NUL): \"100644 blob\"";
    let error = GitError { args: Vec::new(), code: None, stderr: stderr.to_string() };
    assert_eq!(
        path_refused(tree_refusal(TreeWithPathsError::Git(error), &files())),
        (None, "not ls-tree records for the paths asked (no final NUL): \"100644 blob\"".to_string())
    );
}
