use crate::worktree_add_request::worktree_add_request;
use crate::Oid;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

const COMMIT: &str = "489d82745705a031306253da29bdad7aee19d2e5";
const COMMIT256: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn a_branch_is_created_with_dash_b() {
    let (args, _) = worktree_add_request(Path::new("../wt1"), &oid(COMMIT), Some("wt1")).unwrap();
    assert_eq!(
        args,
        argv(&[
            "worktree",
            "add",
            "--no-checkout",
            "-b",
            "wt1",
            "--",
            "../wt1",
            "489d82745705a031306253da29bdad7aee19d2e5",
        ])
    );
}

#[test]
fn no_branch_is_detached() {
    let (args, _) = worktree_add_request(Path::new("../wt2"), &oid(COMMIT256), None).unwrap();
    assert_eq!(
        args,
        argv(&[
            "worktree",
            "add",
            "--no-checkout",
            "--detach",
            "--",
            "../wt2",
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ])
    );
}

#[test]
fn a_non_utf8_path_is_passed_as_its_bytes() {
    let path = Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"));
    let (args, _) = worktree_add_request(path, &oid(COMMIT), None).unwrap();
    assert_eq!(args[5].as_bytes(), b"/tmp/caf\xe9");
}

#[test]
fn a_path_starting_with_a_dash_follows_the_separator() {
    let (args, _) = worktree_add_request(Path::new("-wt"), &oid(COMMIT), None).unwrap();
    assert_eq!(args[4..6], argv(&["--", "-wt"])[..]);
}

#[test]
fn opts_are_the_defaults_and_not_read_only() {
    let (_, opts) = worktree_add_request(Path::new("../wt1"), &oid(COMMIT), Some("wt1")).unwrap();
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.cwd, None);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn empty_path_is_refused() {
    let err = worktree_add_request(Path::new(""), &oid(COMMIT), None).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let path = Path::new(OsStr::from_bytes(b"../w\0t"));
    let err = worktree_add_request(path, &oid(COMMIT), None).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "worktree path holds a NUL: \"../w\\0t\"");
}

#[test]
fn empty_branch_is_refused() {
    let err = worktree_add_request(Path::new("../wt1"), &oid(COMMIT), Some("")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not a usable branch name (empty or starting with '-'): \"\"");
}

#[test]
fn branch_starting_with_a_dash_is_refused() {
    let err = worktree_add_request(Path::new("../wt1"), &oid(COMMIT), Some("-f")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not a usable branch name (empty or starting with '-'): \"-f\"");
}

#[test]
fn nul_in_branch_is_refused_naming_it() {
    let err = worktree_add_request(Path::new("../wt1"), &oid(COMMIT), Some("w\0t")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "branch holds a NUL: \"w\\0t\"");
}
