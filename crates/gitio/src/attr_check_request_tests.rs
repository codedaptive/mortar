use crate::attr_check_request::attr_check_request;
use crate::Oid;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn commit() -> Oid {
    Oid::parse("d00491fd7e5bb6fa28c517a0bb32b8b506539d4d").unwrap()
}

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

#[test]
fn reads_merge_from_the_commit_for_the_paths_after_the_separator() {
    let sent = paths(&["a.txt", "n\nl.txt", "-dash.txt", "sp ace", "./a.txt"]);
    let (args, _) = attr_check_request(&commit(), &sent).unwrap();
    let expected: Vec<OsString> = [
        "--attr-source=d00491fd7e5bb6fa28c517a0bb32b8b506539d4d",
        "check-attr",
        "-z",
        "merge",
        "--",
        "a.txt",
        "n\nl.txt",
        "-dash.txt",
        "sp ace",
        "./a.txt",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn opts_are_read_only_in_the_repo() {
    let (_, opts) = attr_check_request(&commit(), &paths(&["a.txt"])).unwrap();
    assert!(opts.read_only);
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn a_non_utf8_path_goes_into_argv_as_its_bytes() {
    let (args, _) = attr_check_request(&commit(), &[PathBuf::from(OsStr::from_bytes(b"caf\xe9"))]).unwrap();
    assert_eq!(args[5].as_bytes(), b"caf\xe9");
}

#[test]
fn empty_list_is_refused() {
    let err = attr_check_request(&commit(), &[]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "no paths to check");
}

#[test]
fn empty_path_is_refused() {
    let err = attr_check_request(&commit(), &paths(&["a.txt", ""])).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not a usable path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let err = attr_check_request(&commit(), &paths(&["a.txt", "a\0b"])).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "path holds a NUL: \"a\\0b\"");
}
