use crate::worktree_fill_request::worktree_fill_request;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[test]
fn argv_resets_and_writes_head_ignoring_sparse_settings() {
    let (args, _) = worktree_fill_request(Path::new("/tmp/r/wt")).unwrap();
    let want: Vec<OsString> =
        ["read-tree", "--reset", "-u", "--no-sparse-checkout", "HEAD"].map(OsString::from).to_vec();
    assert_eq!(args, want);
}

#[test]
fn opts_write_in_the_folder() {
    let (_, opts) = worktree_fill_request(Path::new("/tmp/r/wt")).unwrap();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let (_, opts) = worktree_fill_request(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))).unwrap();
    assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
}
