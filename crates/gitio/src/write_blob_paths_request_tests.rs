use crate::write_blob_paths_request::write_blob_paths_request;
use crate::Oid;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const TREE: &str = "aaff74984cccd156a469afa7d9ab10e4777beb24";

fn tree() -> Oid {
    Oid::parse(TREE).unwrap()
}

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

#[test]
fn checks_out_the_tree_then_the_paths_after_the_separator() {
    let (args, _) =
        write_blob_paths_request(Path::new("/tmp/w/wt1"), &tree(), &paths(&["f", "d*", "-dash", "n\nl"])).unwrap();
    let expected: Vec<OsString> = [
        "checkout",
        "--end-of-options",
        "aaff74984cccd156a469afa7d9ab10e4777beb24",
        "--",
        "f",
        "d*",
        "-dash",
        "n\nl",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(args, expected);
}

#[test]
fn runs_in_the_folder_with_literal_pathspecs_and_writes() {
    let (_, opts) = write_blob_paths_request(Path::new("/tmp/w/wt1"), &tree(), &paths(&["f"])).unwrap();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/w/wt1")));
    assert_eq!(opts.env, vec![(OsString::from("GIT_LITERAL_PATHSPECS"), OsString::from("1"))]);
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn a_non_utf8_path_goes_into_argv_as_its_bytes() {
    let path = PathBuf::from(OsStr::from_bytes(b"caf\xe9"));
    let (args, _) = write_blob_paths_request(Path::new("/tmp/w/wt1"), &tree(), &[path]).unwrap();
    assert_eq!(args[4].as_bytes(), b"caf\xe9");
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let folder = Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"));
    let (_, opts) = write_blob_paths_request(folder, &tree(), &paths(&["f"])).unwrap();
    assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
}

#[test]
fn empty_list_is_refused() {
    let err = write_blob_paths_request(Path::new("/tmp/w/wt1"), &tree(), &[]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "no paths to write");
}

#[test]
fn empty_folder_is_refused() {
    let err = write_blob_paths_request(Path::new(""), &tree(), &paths(&["f"])).unwrap_err();
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn empty_path_is_refused() {
    let err = write_blob_paths_request(Path::new("/tmp/w/wt1"), &tree(), &paths(&["f", ""])).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not a usable path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let err = write_blob_paths_request(Path::new("/tmp/w/wt1"), &tree(), &paths(&["f", "a\0b"])).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "path holds a NUL: \"a\\0b\"");
}
