//! tree_with_paths_typed's refusals as their TreeWithPathsRefusal variants, and the untyped tree_with_paths' text.
use crate::fixture::{files, repo};
use gitio::{GitError, Oid, TreeWithPathsError, TreeWithPathsRefusal};
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
use TreeWithPathsRefusal::{FolderInBase, FolderInFrom, NotAsStored, Overlap, Unusable};

type Answers = (Result<Oid, TreeWithPathsError>, Result<Oid, GitError>);

/// Both methods' answers for `paths`, with base holding `base-dir/child` and `ordinary`, from `from-dir/child` and
/// `ordinary`.
fn attempt(paths: &[PathBuf]) -> Answers {
    let r = repo();
    let base = files(&r.git, &[("base-dir/child", b"base\n"), ("ordinary", b"base\n")]);
    let from = files(&r.git, &[("from-dir/child", b"from\n"), ("ordinary", b"from\n")]);
    (r.git.tree_with_paths_typed(&base, &from, paths), r.git.tree_with_paths(&base, &from, paths))
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
}

fn refusal(stderr: &str) -> Result<Oid, GitError> {
    Err(GitError { args: Vec::new(), code: None, stderr: stderr.to_string() })
}

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}

#[test]
fn a_folder_in_from_is_refused_as_folder_in_from() {
    let (typed, untyped) = attempt(&paths(&["from-dir"]));
    assert_eq!(typed, Err(TreeWithPathsError::Refused(FolderInFrom { path: path("from-dir") })));
    assert_eq!(untyped, refusal("not a file entry: \"from-dir\" is a folder"));
}

#[test]
fn a_folder_in_base_that_from_lacks_is_refused_as_folder_in_base() {
    let (typed, untyped) = attempt(&paths(&["base-dir"]));
    assert_eq!(typed, Err(TreeWithPathsError::Refused(FolderInBase { path: path("base-dir") })));
    assert_eq!(untyped, refusal("not a file entry: \"base-dir\" is a folder"));
}

#[test]
fn a_folder_and_a_path_under_it_are_refused_as_overlap_naming_both() {
    let (typed, untyped) = attempt(&paths(&["ordinary/child", "ordinary"]));
    let expected = Overlap { folder: path("ordinary"), inner: path("ordinary/child") };
    assert_eq!(typed, Err(TreeWithPathsError::Refused(expected)));
    assert_eq!(untyped, refusal("not usable paths (one is a folder holding the other): \"ordinary\" and \"ordinary/child\""));
}

#[test]
fn each_path_not_written_as_git_stores_it_is_refused_as_not_as_stored() {
    let cases = [
        ("/ordinary", "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"/ordinary\""),
        ("ordinary/", "not a usable path (ends in '/', naming a folder's children): \"ordinary/\""),
        ("a//b", "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a//b\""),
        ("a/./b", "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a/./b\""),
        ("a/../b", "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"a/../b\""),
        ("./ordinary", "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): \"./ordinary\""),
    ];
    for (name, stderr) in cases {
        let (typed, untyped) = attempt(&paths(&[name]));
        assert_eq!(typed, Err(TreeWithPathsError::Refused(NotAsStored { path: path(name) })), "{name}");
        assert_eq!(untyped, refusal(stderr), "{name}");
    }
}

#[test]
fn an_empty_path_is_refused_as_unusable() {
    let (typed, untyped) = attempt(&paths(&[""]));
    assert_eq!(typed, Err(TreeWithPathsError::Refused(Unusable { path: path("") })));
    assert_eq!(untyped, refusal("not a usable path (empty): \"\""));
}

#[test]
fn a_path_holding_a_nul_is_refused_as_unusable() {
    let nul = PathBuf::from(OsString::from_vec(b"has\0nul".to_vec()));
    let (typed, untyped) = attempt(std::slice::from_ref(&nul));
    assert_eq!(typed, Err(TreeWithPathsError::Refused(Unusable { path: nul })));
    // Debug writes the NUL as backslash-zero.
    assert_eq!(untyped, refusal("path holds a NUL: \"has\\0nul\""));
}

#[test]
fn a_missing_base_with_no_paths_is_git_with_an_exit_code() {
    let r = repo();
    let from = files(&r.git, &[("ordinary", b"from\n")]);
    let missing = Oid::parse(&"0".repeat(40)).unwrap();
    let typed = r.git.tree_with_paths_typed(&missing, &from, &[]);
    assert!(matches!(typed, Err(TreeWithPathsError::Git(GitError { code: Some(_), .. }))), "{typed:?}");
}

#[test]
fn a_file_path_takes_from_entry_and_matches_the_untyped_answer() {
    let r = repo();
    let base = files(&r.git, &[("base-dir/child", b"base\n"), ("ordinary", b"base\n")]);
    let from = files(&r.git, &[("from-dir/child", b"from\n"), ("ordinary", b"from\n")]);
    let expected = files(&r.git, &[("base-dir/child", b"base\n"), ("ordinary", b"from\n")]);
    let selected = paths(&["ordinary"]);
    assert_eq!(r.git.tree_with_paths_typed(&base, &from, &selected), Ok(expected.clone()));
    assert_eq!(r.git.tree_with_paths(&base, &from, &selected), Ok(expected));
}
