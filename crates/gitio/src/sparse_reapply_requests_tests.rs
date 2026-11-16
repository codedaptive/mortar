use crate::sparse_reapply_requests::sparse_reapply_requests;
use crate::RunOpts;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn requests() -> [(Vec<OsString>, RunOpts); 2] {
    sparse_reapply_requests(Path::new("/tmp/r/wt")).unwrap()
}

#[test]
fn first_run_reads_the_sparse_checkout_flag_as_a_bool() {
    assert_eq!(requests()[0].0, argv(&["config", "--type=bool", "--get", "core.sparseCheckout"]));
}

#[test]
fn flag_read_is_read_only_in_the_folder_and_takes_exit_1_as_an_answer() {
    let [(_, opts), _] = requests();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(opts.read_only);
    assert_eq!(opts.answer_codes, vec![1]);
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn second_run_is_a_sparse_checkout_reapply() {
    assert_eq!(requests()[1].0, argv(&["sparse-checkout", "reapply"]));
}

#[test]
fn reapply_writes_in_the_folder_and_accepts_only_exit_0() {
    let [_, (_, opts)] = requests();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    let [flag, reapply] = sparse_reapply_requests(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))).unwrap();
    for opts in [flag.1, reapply.1] {
        assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
    }
}

#[test]
fn empty_folder_is_refused() {
    let err = sparse_reapply_requests(Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_folder_is_refused_naming_it() {
    let err = sparse_reapply_requests(Path::new(OsStr::from_bytes(b"/tmp/w\0t"))).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "worktree path holds a NUL: \"/tmp/w\\0t\"");
}
