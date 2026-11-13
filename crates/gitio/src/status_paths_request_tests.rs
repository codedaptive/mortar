use crate::status_paths_request::status_paths_request;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[test]
fn lists_porcelain_v2_nul_ended_every_untracked_file_and_no_renames() {
    let (args, _) = status_paths_request(Path::new("/tmp/r/wt")).unwrap();
    let want: Vec<OsString> = ["status", "--porcelain=v2", "-z", "--untracked-files=all", "--no-renames"]
        .iter()
        .map(OsString::from)
        .collect();
    assert_eq!(args, want);
}

#[test]
fn runs_read_only_in_the_folder() {
    let (_, opts) = status_paths_request(Path::new("/tmp/r/wt")).unwrap();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let (_, opts) = status_paths_request(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))).unwrap();
    assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
}

#[test]
fn empty_folder_is_refused() {
    let err = status_paths_request(Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_folder_is_refused_naming_it() {
    let err = status_paths_request(Path::new(OsStr::from_bytes(b"/tmp/w\0t"))).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "worktree path holds a NUL: \"/tmp/w\\0t\"");
}
