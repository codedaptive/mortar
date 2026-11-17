//! note_finished_move: a verb's report saying first that it finished a stopped integration folder move (S-137).

use super::FinishedMove;
use crate::report::Report;

/// None is `report` unchanged. Some puts one line first, "first finished a stopped move: the integration folder
/// <folder> moved from <old> to <tip>"; exit, outcome, owner, next and fields stay as they were, so the verb's own
/// answer still decides them.
pub fn note_finished_move(report: Report, finished: Option<&FinishedMove>) -> Report {
    let Some(finished) = finished else {
        return report;
    };
    let first = format!(
        "first finished a stopped move: the integration folder {} moved from {} to {}",
        finished.folder.display(),
        finished.old,
        finished.tip
    );
    let mut lines = vec![first];
    lines.extend(report.lines);
    Report { lines, ..report }
}
