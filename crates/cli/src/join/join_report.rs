//! The report of an `mr join` that ran to an answer.

use super::{answer_part, moved_before, save_part, JoinOutcome};
use crate::report::{failure_report, OutValue, Report};

const VERB: &str = "join";

/// Lines: save_part's, then answer_part's, told by moved_before what the save (the request branch) and the snapshot
/// (its ref) moved before the join. Fields: request_ref, save_part's, then answer_part's. Exit, outcome and next are
/// answer_part's. FilesWritten, which only resolve gives, is failure_report.
pub fn join_report(outcome: &JoinOutcome) -> Report {
    let moved = moved_before(outcome.save.written, outcome.snapshot_written, &outcome.request_ref);
    let Some(answer) = answer_part(&outcome.answer, &outcome.request_ref, moved.as_deref()) else {
        return failure_report(VERB, "join answered files-written, which only resolve gives");
    };
    let (mut lines, saved) = save_part(&outcome.save, outcome.finished_move.as_ref());
    lines.extend(answer.lines);
    let mut fields = vec![("request_ref".to_string(), OutValue::Text(outcome.request_ref.clone()))];
    fields.extend(saved);
    fields.extend(answer.fields);
    Report {
        verb: VERB.to_string(),
        exit: answer.exit,
        outcome: answer.outcome.to_string(),
        lines,
        owner: None,
        next: answer.next.to_string(),
        fields,
    }
}
