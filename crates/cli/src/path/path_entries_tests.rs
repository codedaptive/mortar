//! path_entries: a PATH value read as the shell reads it, in lookup order: unset is no entries, an empty or relative
//! entry is under cwd, an absolute one as given. Pure: none of these paths exist.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::path_entries;

const CWD: &str = "/work/cwd";

fn entries(path: Option<&str>) -> Vec<PathBuf> {
    path_entries(path.map(OsStr::new), Path::new(CWD))
}

fn paths(expected: &[&str]) -> Vec<PathBuf> {
    expected.iter().map(PathBuf::from).collect()
}

#[test]
fn unset_gives_no_entries() {
    assert_eq!(entries(None), Vec::<PathBuf>::new());
}

#[test]
fn set_but_empty_gives_cwd() {
    assert_eq!(entries(Some("")), paths(&["/work/cwd"]));
}

#[test]
fn a_leading_empty_entry_is_cwd_first() {
    assert_eq!(entries(Some(":/a")), paths(&["/work/cwd", "/a"]));
}

#[test]
fn a_trailing_empty_entry_is_cwd_last() {
    assert_eq!(entries(Some("/a:")), paths(&["/a", "/work/cwd"]));
}

#[test]
fn a_relative_entry_is_under_cwd() {
    assert_eq!(entries(Some("sub:/a")), paths(&["/work/cwd/sub", "/a"]));
}

#[test]
fn an_empty_entry_between_two_is_cwd_in_place() {
    assert_eq!(entries(Some("/a::/b")), paths(&["/a", "/work/cwd", "/b"]));
}

#[test]
fn an_absolute_entry_is_unchanged() {
    assert_eq!(entries(Some("/usr/local/bin")), paths(&["/usr/local/bin"]));
}
