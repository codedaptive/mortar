//! Where the report is written.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// `ACCEPT_REPORT` when set and not empty; else `<CARGO_TARGET_DIR>/accept/report.txt` when that is set and not
/// empty (the spec's `target/` is cargo's build output, which codeteam moves out of `target/`); else
/// `<workspace>/target/accept/report.txt`. A relative value is taken from the workspace root, because cargo runs
/// the test in crates/cli while make and the reader work from the root.
pub fn report_path(accept_report: Option<&OsStr>, cargo_target_dir: Option<&OsStr>, workspace: &Path) -> PathBuf {
    let set = |value: Option<&OsStr>| value.filter(|value| !value.is_empty()).map(|value| workspace.join(value));
    set(accept_report)
        .or_else(|| set(cargo_target_dir).map(|target| target.join("accept/report.txt")))
        .unwrap_or_else(|| workspace.join("target/accept/report.txt"))
}
