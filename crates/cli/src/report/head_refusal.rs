//! head_refusal: the refusal of a verb whose folder's HEAD is unborn or detached, which no repeat can pass until HEAD
//! is mended, except an attempt folder's unborn HEAD in mr folder keep and drop, whose repeat re-makes the orphan
//! (folder/attempt_snapshot_failure.rs). It is the one source of that refusal's exit, outcome and owner for snapshot
//! and save (save/snapshot_error_report.rs, save/save_error_report.rs), for attempt folders in mr folder keep and
//! drop, for status (status/saved_state_report.rs; disk keeps only its lines), and, through repeat_head_refusal,
//! for the rest of its users; it sits in report/, the lowest level covering them all.

use super::{ExitCode, Report};

/// Exit Refused, outcome "refused", owner the folder's HEAD, under `verb`, with `lines` and `next` as the caller
/// gives them: save, an attempt folder's snapshot and status in an attempt folder (naming the try branch)
/// name their own next, the rest take repeat_head_refusal's.
pub(crate) fn head_refusal(verb: &str, lines: Vec<String>, next: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some("the folder's HEAD".to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}
