use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::report_path;

const WORKSPACE: &str = "/ws/mortar";

fn path(accept_report: Option<&str>, cargo_target_dir: Option<&str>) -> PathBuf {
    report_path(accept_report.map(OsStr::new), cargo_target_dir.map(OsStr::new), Path::new(WORKSPACE))
}

#[test]
#[ignore = "acceptance: make accept"]
fn accept_report_wins_and_a_relative_one_is_under_the_workspace() {
    assert_eq!(path(Some("/out/r.txt"), Some("/ct")), PathBuf::from("/out/r.txt"));
    assert_eq!(path(Some("out/r.txt"), None), PathBuf::from("/ws/mortar/out/r.txt"));
}

#[test]
#[ignore = "acceptance: make accept"]
fn cargo_target_dir_when_accept_report_is_unset_or_empty() {
    assert_eq!(path(None, Some("/ws/mortar/cargo-target")), PathBuf::from("/ws/mortar/cargo-target/accept/report.txt"));
    assert_eq!(path(Some(""), Some("ct")), PathBuf::from("/ws/mortar/ct/accept/report.txt"));
}

#[test]
#[ignore = "acceptance: make accept"]
fn workspace_target_when_neither_is_set() {
    assert_eq!(path(None, None), PathBuf::from("/ws/mortar/target/accept/report.txt"));
    assert_eq!(path(Some(""), Some("")), PathBuf::from("/ws/mortar/target/accept/report.txt"));
}
