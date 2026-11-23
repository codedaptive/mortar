//! The report of an `mr commit-files` in a zone folder that found the named paths already held while a stopped run
//! left them staged in the folder's index. same_staged_report_tests covers it.

use std::path::PathBuf;

use gitio::Oid;

use super::commit_files_report;
use crate::report::{OutValue, Report};

/// commit_files_report's "same" for `refname` at `old` (its first line and the fields commit, ref, moved and paths
/// unchanged), with a second line naming the paths a stopped run left staged, a field staged listing them after paths,
/// and next `git restore --staged -- <paths>`. The stopped run's named paths are still staged, so the index is not at
/// the tip and the pending marker is kept; restoring them to HEAD, which is zone/<zone>'s tip in a zone folder, lets
/// the next run find the index at the tip and remove the marker.
pub fn same_staged_report(refname: &str, old: &Oid, paths: &[PathBuf]) -> Report {
    let shown: Vec<String> = paths.iter().map(|path| path.display().to_string()).collect();
    let mut report =
        commit_files_report(refname, old, None, paths, format!("git restore --staged -- {}", shown.join(" ")));
    report.lines.push(format!(
        "a stopped mr commit-files left the named paths staged in the folder's index: {}",
        shown.join(", ")
    ));
    report.fields.push(("staged".to_string(), OutValue::Texts(shown)));
    report
}
