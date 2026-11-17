//! The files a re-review's range-diff shows a change in (check 9).

use super::rereview_changes;

/// The sections of rereview_changes(`lines`) that are files, sorted and once each: changes in the metadata or the
/// message, and any before a section starts, are not files and are left out.
pub fn rereview_files(lines: &[String]) -> Vec<String> {
    let mut files: Vec<String> = rereview_changes(lines)
        .into_iter()
        .map(|(section, _)| section)
        .filter(|section| !["", "Metadata", "Commit message"].contains(&section.as_str()))
        .collect();
    files.sort();
    files.dedup();
    files
}
