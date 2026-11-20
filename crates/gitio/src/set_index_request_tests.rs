use crate::set_index_request::set_index_request;
use crate::Oid;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const SHA1: &str = "d00491fd7e5bb6fa28c517a0bb32b8b506539d4d";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

#[test]
fn reads_a_sha1_tree_into_the_index_with_no_merge_or_update() {
    let (args, _) = set_index_request(Path::new("/tmp/r/wt"), &oid(SHA1)).unwrap();
    assert_eq!(args, argv(&["read-tree", "--end-of-options", "d00491fd7e5bb6fa28c517a0bb32b8b506539d4d"]));
}

#[test]
fn reads_a_sha256_tree_into_the_index() {
    let (args, _) = set_index_request(Path::new("/tmp/r/wt"), &oid(SHA256)).unwrap();
    assert_eq!(
        args,
        argv(&[
            "read-tree",
            "--end-of-options",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ])
    );
}

#[test]
fn writes_the_real_index_in_the_folder_and_accepts_only_exit_0() {
    let (_, opts) = set_index_request(Path::new("/tmp/r/wt"), &oid(SHA1)).unwrap();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let (_, opts) = set_index_request(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9")), &oid(SHA1)).unwrap();
    assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
}

#[test]
fn empty_folder_is_refused() {
    let err = set_index_request(Path::new(""), &oid(SHA1)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_folder_is_refused_naming_it() {
    let err = set_index_request(Path::new(OsStr::from_bytes(b"/tmp/w\0t")), &oid(SHA1)).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "worktree path holds a NUL: \"/tmp/w\\0t\"");
}
