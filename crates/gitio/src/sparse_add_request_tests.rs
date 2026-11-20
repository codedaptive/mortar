use crate::sparse_add_request::sparse_add_request;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn paths(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn adds_the_paths_after_the_separator() {
    let (args, _) = sparse_add_request(Path::new("/tmp/c/wt1"), &paths(&["d2", "sp ace", "-dash"])).unwrap();
    assert_eq!(args, argv(&["sparse-checkout", "add", "--", "d2", "sp ace", "-dash"]));
}

#[test]
fn runs_in_the_folder_and_writes() {
    let (_, opts) = sparse_add_request(Path::new("/tmp/c/wt1"), &paths(&["d2"])).unwrap();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/c/wt1")));
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let (_, opts) = sparse_add_request(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9")), &paths(&["d2"])).unwrap();
    assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
}

#[test]
fn empty_list_is_refused() {
    let err = sparse_add_request(Path::new("/tmp/c/wt1"), &[]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "no sparse paths to add");
}

#[test]
fn empty_folder_is_refused() {
    let err = sparse_add_request(Path::new(""), &paths(&["d2"])).unwrap_err();
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn empty_path_is_refused() {
    let err = sparse_add_request(Path::new("/tmp/c/wt1"), &paths(&["d2", ""])).unwrap_err();
    assert_eq!(err.stderr, "not a usable sparse path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let err = sparse_add_request(Path::new("/tmp/c/wt1"), &paths(&["a\0b"])).unwrap_err();
    assert_eq!(err.stderr, "sparse path holds a NUL: \"a\\0b\"");
}

#[test]
fn line_break_in_path_is_refused_naming_it() {
    let err = sparse_add_request(Path::new("/tmp/c/wt1"), &paths(&["d2", "n\nl"])).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "sparse path holds a line break: \"n\\nl\"");
}
