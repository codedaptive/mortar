//! Whether a re-review's range-diff changes one file by exactly one line edit (check 9).

use super::rereview_changes;

/// True when the changed lines of rereview_changes(`lines`) in the section `file` are exactly, in order, `-+<removed>`
/// (the line the first squash added and the second does not) and `++<added>` (the line the second adds instead):
/// any other line of `file` marked changed, or either of the two missing, is false. Other sections are not read.
pub fn rereview_only_edit(lines: &[String], file: &str, removed: &str, added: &str) -> bool {
    let changed: Vec<String> =
        rereview_changes(lines).into_iter().filter(|(section, _)| section == file).map(|(_, line)| line).collect();
    changed == [format!("-+{removed}"), format!("++{added}")]
}
