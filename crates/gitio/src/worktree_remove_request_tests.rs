use crate::worktree_remove_request::worktree_remove_request;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn without_force_the_path_follows_the_separator() {
    let (args, _) = worktree_remove_request(Path::new("../wt1"), false).unwrap();
    assert_eq!(args, argv(&["worktree", "remove", "--", "../wt1"]));
}

#[test]
fn force_adds_the_flag_before_the_separator() {
    let (args, _) = worktree_remove_request(Path::new("/tmp/c/wt2"), true).unwrap();
    assert_eq!(args, argv(&["worktree", "remove", "--force", "--", "/tmp/c/wt2"]));
}

#[test]
fn a_non_utf8_path_is_passed_as_its_bytes() {
    let (args, _) = worktree_remove_request(Path::new(OsStr::from_bytes(b"/tmp/\xff\xfe wt")), false).unwrap();
    assert_eq!(args[3].as_bytes(), b"/tmp/\xff\xfe wt");
}

#[test]
fn opts_are_the_defaults_and_not_read_only() {
    let (_, opts) = worktree_remove_request(Path::new("../wt1"), true).unwrap();
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.cwd, None);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn empty_path_is_refused() {
    let err = worktree_remove_request(Path::new(""), true).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let err = worktree_remove_request(Path::new(OsStr::from_bytes(b"wt\0")), false).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "worktree path holds a NUL: \"wt\\0\"");
}
