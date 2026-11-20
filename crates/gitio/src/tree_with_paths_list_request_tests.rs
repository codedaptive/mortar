use crate::tree_with_paths_list_request::tree_with_paths_list_request;
use crate::{Oid, TreeWithPathsError, TreeWithPathsRefusal};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn from() -> Oid {
    Oid::parse("aaff74984cccd156a469afa7d9ab10e4777beb24").unwrap()
}

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

fn refused(items: Vec<PathBuf>) -> TreeWithPathsRefusal {
    match tree_with_paths_list_request(&from(), &items) {
        Err(TreeWithPathsError::Refused(refusal)) => refusal,
        other => panic!("expected a refusal: {other:?}"),
    }
}

#[test]
fn lists_every_path_from_the_tree_root_nul_ended_without_precomposition() {
    let (args, _) = tree_with_paths_list_request(&from(), &paths(&["a/f", "-dash", "g"])).unwrap().unwrap();
    let expected: Vec<OsString> = [
        "-c",
        "core.precomposeunicode=false",
        "ls-tree",
        "-z",
        "--full-tree",
        "--end-of-options",
        "aaff74984cccd156a469afa7d9ab10e4777beb24",
        "--",
        "a/f",
        "-dash",
        "g",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_are_read_only_literal_pathspecs_in_the_repo() {
    let (_, opts) = tree_with_paths_list_request(&from(), &paths(&["f"])).unwrap().unwrap();
    assert!(opts.read_only);
    assert_eq!(opts.env, vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("1"))]);
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn no_paths_is_no_run() {
    assert!(tree_with_paths_list_request(&from(), &[]).unwrap().is_none());
}

#[test]
fn a_path_holding_a_newline_or_starting_with_a_colon_passes_as_bytes() {
    let (args, _) = tree_with_paths_list_request(&from(), &paths(&["n\nl", ":x", "t\tb"])).unwrap().unwrap();
    assert_eq!(args[8..], [OsString::from("n\nl"), OsString::from(":x"), OsString::from("t\tb")]);
}

#[test]
fn a_non_utf8_path_goes_into_argv_as_its_bytes() {
    let items = vec![PathBuf::from(OsStr::from_bytes(b"caf\xe9"))];
    let (args, _) = tree_with_paths_list_request(&from(), &items).unwrap().unwrap();
    assert_eq!(args[8].as_bytes(), b"caf\xe9");
}

#[test]
fn an_empty_path_is_unusable() {
    assert_eq!(refused(paths(&["ok", ""])), TreeWithPathsRefusal::Unusable { path: PathBuf::from("") });
}

#[test]
fn a_nul_in_a_path_is_unusable_naming_it() {
    let items = vec![PathBuf::from("ok"), PathBuf::from(OsStr::from_bytes(b"a\0b"))];
    assert_eq!(refused(items), TreeWithPathsRefusal::Unusable { path: PathBuf::from(OsStr::from_bytes(b"a\0b")) });
}

#[test]
fn a_path_ending_in_a_slash_is_not_as_stored_naming_it() {
    assert_eq!(refused(paths(&["ok", "dir/"])), TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("dir/") });
}

#[test]
fn a_path_not_as_git_stores_it_is_not_as_stored_naming_it() {
    for path in ["/a", "./a", "a//b", "a/./b", "a/../b", ".."] {
        assert_eq!(refused(paths(&["ok", path])), TreeWithPathsRefusal::NotAsStored { path: PathBuf::from(path) });
    }
}

#[test]
fn an_unusable_path_is_refused_before_a_later_not_as_stored_one() {
    assert_eq!(refused(paths(&["", "/a"])), TreeWithPathsRefusal::Unusable { path: PathBuf::from("") });
}

#[test]
fn a_bad_path_is_refused_before_an_overlap() {
    assert_eq!(
        refused(paths(&["f", "f/x", "a//b"])),
        TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("a//b") }
    );
}

#[test]
fn a_folder_and_a_path_under_it_overlap_naming_both() {
    assert_eq!(
        refused(paths(&["f", "g", "f/x"])),
        TreeWithPathsRefusal::Overlap { folder: PathBuf::from("f"), inner: PathBuf::from("f/x") }
    );
}

#[test]
fn a_path_under_a_folder_given_first_overlaps_naming_the_folder_first() {
    assert_eq!(
        refused(paths(&["f/x", "f"])),
        TreeWithPathsRefusal::Overlap { folder: PathBuf::from("f"), inner: PathBuf::from("f/x") }
    );
}

#[test]
fn a_folder_and_a_path_deep_under_it_overlap() {
    assert_eq!(
        refused(paths(&["a", "a/b/c"])),
        TreeWithPathsRefusal::Overlap { folder: PathBuf::from("a"), inner: PathBuf::from("a/b/c") }
    );
}

#[test]
fn a_name_extending_another_without_a_slash_passes() {
    let (args, _) = tree_with_paths_list_request(&from(), &paths(&["a", "ab"])).unwrap().unwrap();
    assert_eq!(args[8..], [OsString::from("a"), OsString::from("ab")]);
}

#[test]
fn the_same_path_twice_passes() {
    let (args, _) = tree_with_paths_list_request(&from(), &paths(&["a/b", "a/b"])).unwrap().unwrap();
    assert_eq!(args[8..], [OsString::from("a/b"), OsString::from("a/b")]);
}
