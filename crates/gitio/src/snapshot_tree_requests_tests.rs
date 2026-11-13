use crate::snapshot_tree_requests::snapshot_tree_requests;
use crate::{Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

const BASE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const INDEX: &str = "/tmp/mortar-index-7-99-0";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn excludes(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn requests(items: &[&str]) -> [(Vec<OsString>, RunOpts); 3] {
    snapshot_tree_requests(&oid(BASE), &excludes(items), Path::new(INDEX)).unwrap()
}

#[test]
fn first_run_reads_the_base_tree() {
    assert_eq!(
        requests(&[])[0].0,
        argv(&["read-tree", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"])
    );
}

#[test]
fn second_run_adds_everything_with_excludes_as_pathspecs() {
    assert_eq!(
        requests(&["a/build.o", "target dir"])[1].0,
        argv(&["add", "-A", "--", ".", ":!a/build.o", ":!target dir"])
    );
}

#[test]
fn no_excludes_adds_everything() {
    assert_eq!(requests(&[])[1].0, argv(&["add", "-A", "--", "."]));
}

#[test]
fn third_run_writes_the_tree() {
    assert_eq!(requests(&[])[2].0, argv(&["write-tree"]));
}

#[test]
fn every_run_uses_the_temp_index_only_and_writes() {
    for (_, opts) in requests(&["x"]) {
        assert_eq!(
            opts.env,
            vec![(OsString::from("GIT_INDEX_FILE"), OsString::from("/tmp/mortar-index-7-99-0"))]
        );
        assert!(!opts.read_only);
        assert_eq!(opts.stdin, None);
        assert!(opts.answer_codes.is_empty());
        assert_eq!(opts.cwd, None);
    }
}

#[test]
fn empty_exclude_is_refused() {
    let err = snapshot_tree_requests(&oid(BASE), &excludes(&["ok", ""]), Path::new(INDEX)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn nul_in_exclude_is_refused_naming_it() {
    let err = snapshot_tree_requests(&oid(BASE), &excludes(&["a\0b"]), Path::new(INDEX)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"a\\0b\""), "{}", err.stderr);
}
