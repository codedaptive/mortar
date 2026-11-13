use crate::sparse_list_request::sparse_list_request;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[test]
fn lists_with_cone_mode_forced() {
    let (args, _) = sparse_list_request(Path::new("/tmp/c/wt1")).unwrap();
    let want: Vec<OsString> =
        ["-c", "core.sparseCheckoutCone=true", "sparse-checkout", "list"].iter().map(OsString::from).collect();
    assert_eq!(args, want);
}

#[test]
fn runs_read_only_in_the_folder() {
    let (_, opts) = sparse_list_request(Path::new("/tmp/c/wt1")).unwrap();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/c/wt1")));
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let (_, opts) = sparse_list_request(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))).unwrap();
    assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
}

#[test]
fn empty_folder_is_refused() {
    let err = sparse_list_request(Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_folder_is_refused_naming_it() {
    let err = sparse_list_request(Path::new(OsStr::from_bytes(b"/tmp/w\0t"))).unwrap_err();
    assert_eq!(err.stderr, "worktree path holds a NUL: \"/tmp/w\\0t\"");
}
