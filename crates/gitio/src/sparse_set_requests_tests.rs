use crate::sparse_set_requests::sparse_set_requests;
use crate::RunOpts;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn paths(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn requests(items: &[&str]) -> [(Vec<OsString>, RunOpts); 2] {
    sparse_set_requests(Path::new("/tmp/c/wt1"), &paths(items)).unwrap()
}

#[test]
fn first_run_sets_the_cone_with_the_paths_after_the_separator() {
    assert_eq!(
        requests(&["a", "b dir", "-c"])[0].0,
        argv(&["sparse-checkout", "set", "--cone", "--", "a", "b dir", "-c"])
    );
}

#[test]
fn no_paths_sets_an_empty_cone() {
    assert_eq!(requests(&[])[0].0, argv(&["sparse-checkout", "set", "--cone", "--"]));
}

#[test]
fn second_run_is_a_bare_checkout() {
    assert_eq!(requests(&["a"])[1].0, argv(&["checkout"]));
}

#[test]
fn both_runs_are_in_the_folder_and_write() {
    for (_, opts) in requests(&["a"]) {
        assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/c/wt1")));
        assert!(!opts.read_only);
        assert!(opts.answer_codes.is_empty());
        assert_eq!(opts.stdin, None);
        assert!(opts.env.is_empty());
    }
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let folder = Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"));
    let [set, checkout] = sparse_set_requests(folder, &paths(&["a"])).unwrap();
    for opts in [set.1, checkout.1] {
        assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
    }
}

#[test]
fn empty_path_is_refused() {
    let err = sparse_set_requests(Path::new("/tmp/c/wt1"), &paths(&["a", ""])).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable sparse path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let err = sparse_set_requests(Path::new("/tmp/c/wt1"), &paths(&["a\0b"])).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "sparse path holds a NUL: \"a\\0b\"");
}

#[test]
fn line_break_in_path_is_refused_naming_it() {
    let err = sparse_set_requests(Path::new("/tmp/c/wt1"), &paths(&["a\nb"])).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "sparse path holds a line break: \"a\\nb\"");
}
