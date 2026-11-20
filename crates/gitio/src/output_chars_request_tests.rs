use crate::output_chars_request::output_chars_request;
use crate::BaselineQuery;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn args(query: &BaselineQuery) -> Vec<OsString> {
    output_chars_request(Path::new("/tmp/r/wt"), query).unwrap().0
}

fn os(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn two_paths() -> Vec<PathBuf> {
    vec![PathBuf::from("src/*.rs"), PathBuf::from("-dash file")]
}

#[test]
fn log_patches_argv() {
    let query = BaselineQuery::LogPatches { range: "a1..b2".into(), paths: vec![] };
    let want = ["--no-pager", "-c", "color.ui=never", "log", "-p", "--no-color", "--no-ext-diff", "a1..b2", "--"];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn log_patches_argv_with_paths() {
    let query = BaselineQuery::LogPatches { range: "main~3..main".into(), paths: two_paths() };
    let want = [
        "--no-pager", "-c", "color.ui=never", "log", "-p", "--no-color", "--no-ext-diff", "main~3..main", "--",
        "src/*.rs", "-dash file",
    ];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn diff_argv() {
    let query = BaselineQuery::Diff { a: "HEAD~1".into(), b: "HEAD".into(), paths: vec![] };
    let want = ["--no-pager", "-c", "color.ui=never", "diff", "--no-color", "--no-ext-diff", "HEAD~1", "HEAD", "--"];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn diff_argv_with_paths() {
    let query = BaselineQuery::Diff { a: "v1".into(), b: "v2".into(), paths: two_paths() };
    let want = [
        "--no-pager", "-c", "color.ui=never", "diff", "--no-color", "--no-ext-diff", "v1", "v2", "--", "src/*.rs",
        "-dash file",
    ];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn diff_head_argv() {
    let query = BaselineQuery::DiffHead { paths: vec![] };
    let want = ["--no-pager", "-c", "color.ui=never", "diff", "--no-color", "--no-ext-diff", "HEAD", "--"];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn diff_head_argv_with_paths() {
    let query = BaselineQuery::DiffHead { paths: two_paths() };
    let want = [
        "--no-pager", "-c", "color.ui=never", "diff", "--no-color", "--no-ext-diff", "HEAD", "--", "src/*.rs",
        "-dash file",
    ];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn status_argv() {
    let query = BaselineQuery::Status { paths: vec![] };
    let want = ["--no-pager", "-c", "color.ui=never", "-c", "color.status=never", "status", "--long", "--"];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn status_argv_with_paths() {
    let query = BaselineQuery::Status { paths: two_paths() };
    let want = [
        "--no-pager", "-c", "color.ui=never", "-c", "color.status=never", "status", "--long", "--", "src/*.rs",
        "-dash file",
    ];
    assert_eq!(args(&query), os(&want));
}

#[test]
fn a_non_utf8_path_is_kept_as_its_bytes() {
    let query = BaselineQuery::DiffHead { paths: vec![PathBuf::from(OsStr::from_bytes(b"caf\xe9.txt"))] };
    assert_eq!(args(&query).last().unwrap().as_bytes(), b"caf\xe9.txt");
}

#[test]
fn runs_read_only_in_the_folder() {
    let (_, opts) = output_chars_request(Path::new("/tmp/r/wt"), &BaselineQuery::Status { paths: vec![] }).unwrap();
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
}

fn refused(folder: &str, query: &BaselineQuery) -> String {
    let err = output_chars_request(Path::new(folder), query).unwrap_err();
    assert_eq!(err.code, None, "{query:?}");
    assert!(err.args.is_empty(), "{query:?}");
    err.stderr
}

#[test]
fn an_unusable_range_is_refused() {
    let log = |range: &str| BaselineQuery::LogPatches { range: range.into(), paths: vec![] };
    assert_eq!(refused("/w", &log("")), "not a usable revision range: \"\"");
    assert_eq!(refused("/w", &log("--output=x")), "not a usable revision range: \"--output=x\"");
    assert_eq!(refused("/w", &log("HEAD:file")), "not a usable revision range: \"HEAD:file\"");
}

#[test]
fn an_unusable_revision_on_either_side_is_refused() {
    let diff = |a: &str, b: &str| BaselineQuery::Diff { a: a.into(), b: b.into(), paths: vec![] };
    assert_eq!(refused("/w", &diff("", "HEAD")), "not a usable revision: \"\"");
    assert_eq!(refused("/w", &diff("-p", "HEAD")), "not a usable revision: \"-p\"");
    assert_eq!(refused("/w", &diff("HEAD", ":x")), "not a usable revision: \":x\"");
    assert_eq!(refused("/w", &diff("HEAD", "-R")), "not a usable revision: \"-R\"");
}

#[test]
fn an_empty_path_is_refused_for_every_kind() {
    let empty = || vec![PathBuf::from("ok"), PathBuf::new()];
    let queries = [
        BaselineQuery::LogPatches { range: "HEAD".into(), paths: empty() },
        BaselineQuery::Diff { a: "HEAD".into(), b: "HEAD".into(), paths: empty() },
        BaselineQuery::DiffHead { paths: empty() },
        BaselineQuery::Status { paths: empty() },
    ];
    for query in &queries {
        assert_eq!(refused("/w", query), "not a usable path (empty): \"\"");
    }
}

#[test]
fn an_empty_folder_is_refused() {
    let stderr = refused("", &BaselineQuery::DiffHead { paths: vec![] });
    assert_eq!(stderr, "not a usable worktree path (empty): \"\"");
}
