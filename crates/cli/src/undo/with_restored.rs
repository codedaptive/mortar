//! with_restored: an `mr undo` report of a resolve step with the paused join that restore_join_state put back
//! (S-148, S-155).

use ::join::RestoredJoin;

use crate::report::{OutValue, Report};

/// `report` with lines naming the commit the paused join stopped at (put back now, or already put back by an
/// earlier run) and, when there are any, its conflicted files; and the fields paused_commit and conflicted. Exit,
/// outcome, next and owner are unchanged.
pub fn with_restored(mut report: Report, restored: &RestoredJoin) -> Report {
    let (paused, said) = match restored {
        RestoredJoin::Restored { paused, .. } => (paused, "the paused join is put back"),
        RestoredJoin::AlreadyRestored { paused, .. } => (paused, "the paused join was already put back"),
    };
    let files: Vec<String> = paused.files.iter().map(|file| file.display().to_string()).collect();
    report.lines.push(format!("{said}: it stopped at commit {}", paused.commit));
    if !files.is_empty() {
        report.lines.push(format!("conflicted files: {}", files.join(", ")));
    }
    report.fields.push(("paused_commit".to_string(), OutValue::Text(paused.commit.to_string())));
    report.fields.push(("conflicted".to_string(), OutValue::Texts(files)));
    report
}
