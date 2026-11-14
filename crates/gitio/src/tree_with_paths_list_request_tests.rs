use crate::tree_with_paths_list_request::tree_with_paths_list_request;
use crate::Oid;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn from() -> Oid {
    Oid::parse("aaff74984cccd156a469afa7d9ab10e4777beb24").unwrap()
}

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

fn refused(items: Vec<PathBuf>) -> String {
    let err = tree_with_paths_list_request(&from(), &items).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
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
fn an_empty_path_is_refused() {
    assert_eq!(refused(paths(&["ok", ""])), "not a usable path (empty): \"\"");
}

#[test]
fn a_nul_in_a_path_is_refused_naming_it() {
    let items = vec![PathBuf::from("ok"), PathBuf::from(OsStr::from_bytes(b"a\0b"))];
    assert_eq!(refused(items), "path holds a NUL: \"a\\0b\"");
}

#[test]
fn a_path_ending_in_a_slash_is_refused_naming_it() {
    assert_eq!(
        refused(paths(&["ok", "dir/"])),
        "not a usable path (ends in '/', naming a folder's children): \"dir/\""
    );
}

const NOT_STORED: &str = "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): ";

#[test]
fn a_path_not_as_git_stores_it_is_refused_naming_it() {
    for (path, named) in [
        ("/a", "\"/a\""),
        ("./a", "\"./a\""),
        ("a//b", "\"a//b\""),
        ("a/./b", "\"a/./b\""),
        ("a/../b", "\"a/../b\""),
        ("..", "\"..\""),
    ] {
        assert_eq!(refused(paths(&["ok", path])), format!("{NOT_STORED}{named}"));
    }
}

#[test]
fn a_folder_and_a_path_under_it_are_refused_naming_both() {
    assert_eq!(
        refused(paths(&["f", "g", "f/x"])),
        "not usable paths (one is a folder holding the other): \"f\" and \"f/x\""
    );
}

#[test]
fn a_path_under_a_folder_given_first_is_refused_naming_both() {
    assert_eq!(
        refused(paths(&["f/x", "f"])),
        "not usable paths (one is a folder holding the other): \"f\" and \"f/x\""
    );
}

#[test]
fn a_folder_and_a_path_deep_under_it_are_refused() {
    assert_eq!(
        refused(paths(&["a", "a/b/c"])),
        "not usable paths (one is a folder holding the other): \"a\" and \"a/b/c\""
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
