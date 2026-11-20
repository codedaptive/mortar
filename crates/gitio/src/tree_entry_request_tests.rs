use crate::tree_entry_request::tree_entry_request;
use crate::Oid;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

fn tree() -> Oid {
    Oid::parse("aaff74984cccd156a469afa7d9ab10e4777beb24").unwrap()
}

#[test]
fn lists_the_path_from_the_tree_root_nul_ended() {
    let (args, _) = tree_entry_request(&tree(), Path::new("-dash")).unwrap();
    let expected: Vec<OsString> = [
        "ls-tree",
        "-z",
        "--full-tree",
        "--end-of-options",
        "aaff74984cccd156a469afa7d9ab10e4777beb24",
        "--",
        "-dash",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_are_read_only_in_the_repo() {
    let (_, opts) = tree_entry_request(&tree(), Path::new("f")).unwrap();
    assert!(opts.read_only);
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn the_path_is_taken_literally_never_as_pathspec_magic() {
    let (args, opts) = tree_entry_request(&tree(), Path::new(":x")).unwrap();
    assert_eq!(opts.env, vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("1"))]);
    assert_eq!(args[6], OsString::from(":x"));
}

#[test]
fn a_non_utf8_path_goes_into_argv_as_its_bytes() {
    let (args, _) = tree_entry_request(&tree(), Path::new(OsStr::from_bytes(b"caf\xe9"))).unwrap();
    assert_eq!(args[6].as_bytes(), b"caf\xe9");
}

#[test]
fn a_path_holding_a_newline_is_kept() {
    let (args, _) = tree_entry_request(&tree(), Path::new("n\nl")).unwrap();
    assert_eq!(args[6], OsString::from("n\nl"));
}

#[test]
fn empty_path_is_refused() {
    let err = tree_entry_request(&tree(), Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let err = tree_entry_request(&tree(), Path::new(OsStr::from_bytes(b"a\0b"))).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "path holds a NUL: \"a\\0b\"");
}
