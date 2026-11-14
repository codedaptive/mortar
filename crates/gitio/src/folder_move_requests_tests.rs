use crate::folder_move_requests::folder_move_requests;
use crate::{Oid, RunOpts};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const OLD: &str = "d00491fd7e5bb6fa28c517a0bb32b8b506539d4d";
const NEW: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn requests(folder: &Path) -> [(Vec<OsString>, RunOpts); 3] {
    folder_move_requests(folder, &oid(OLD), &oid(NEW)).unwrap()
}

#[test]
fn first_run_refreshes_the_index_quietly_even_when_unmerged() {
    assert_eq!(requests(Path::new("/tmp/r/wt"))[0].0, argv(&["update-index", "-q", "--unmerged", "--refresh"]));
}

#[test]
fn second_run_is_a_dry_run_read_tree_with_sparse_checkout_off() {
    assert_eq!(
        requests(Path::new("/tmp/r/wt"))[1].0,
        argv(&[
            "read-tree",
            "--no-sparse-checkout",
            "-n",
            "-m",
            "-u",
            "--end-of-options",
            "d00491fd7e5bb6fa28c517a0bb32b8b506539d4d",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ])
    );
}

#[test]
fn third_run_is_a_two_tree_read_tree_old_then_new() {
    assert_eq!(
        requests(Path::new("/tmp/r/wt"))[2].0,
        argv(&[
            "read-tree",
            "-m",
            "-u",
            "--end-of-options",
            "d00491fd7e5bb6fa28c517a0bb32b8b506539d4d",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ])
    );
}

#[test]
fn refresh_writes_in_the_folder_and_accepts_only_exit_0() {
    let [(_, opts), _, _] = requests(Path::new("/tmp/r/wt"));
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn check_is_read_only_in_the_folder_and_takes_exit_128_as_an_answer() {
    let [_, (_, opts), _] = requests(Path::new("/tmp/r/wt"));
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(opts.read_only);
    assert_eq!(opts.answer_codes, vec![128]);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn read_tree_writes_in_the_folder_and_takes_exit_128_as_an_answer() {
    let [_, _, (_, opts)] = requests(Path::new("/tmp/r/wt"));
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(!opts.read_only);
    assert_eq!(opts.answer_codes, vec![128]);
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    for (_, opts) in requests(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))) {
        assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
    }
}

#[test]
fn empty_folder_is_refused() {
    let err = folder_move_requests(Path::new(""), &oid(OLD), &oid(NEW)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_folder_is_refused_naming_it() {
    let err = folder_move_requests(Path::new(OsStr::from_bytes(b"/tmp/w\0t")), &oid(OLD), &oid(NEW)).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "worktree path holds a NUL: \"/tmp/w\\0t\"");
}
