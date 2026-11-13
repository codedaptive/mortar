use crate::head_requests::head_requests;
use crate::RunOpts;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn requests() -> [(Vec<OsString>, RunOpts); 2] {
    head_requests(Path::new("/tmp/r/wt")).unwrap()
}

#[test]
fn first_run_reads_the_branch_quietly() {
    assert_eq!(requests()[0].0, argv(&["symbolic-ref", "-q", "HEAD"]));
}

#[test]
fn second_run_verifies_head_as_a_commit() {
    assert_eq!(requests()[1].0, argv(&["rev-parse", "--verify", "-q", "--end-of-options", "HEAD^{commit}"]));
}

#[test]
fn both_runs_are_read_only_in_the_folder_with_exit_1_an_answer() {
    for (_, opts) in requests() {
        assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
        assert!(opts.read_only);
        assert_eq!(opts.answer_codes, vec![1]);
        assert_eq!(opts.stdin, None);
        assert!(opts.env.is_empty());
    }
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes() {
    for (_, opts) in head_requests(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))).unwrap() {
        assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
    }
}

#[test]
fn empty_folder_is_refused() {
    let err = head_requests(Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_folder_is_refused_naming_it() {
    let err = head_requests(Path::new(OsStr::from_bytes(b"/tmp/w\0t"))).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "worktree path holds a NUL: \"/tmp/w\\0t\"");
}
