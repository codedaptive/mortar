//! head_refusal: the refusal of a verb whose folder's HEAD is unborn or detached, which no repeat can pass until HEAD
//! is mended. It is the one source of that refusal's owner and next for snapshot (save/snapshot_error_report.rs) and
//! for join and resolve (through snapshot_step_failure), and sits in report/, the lowest level covering both.

use super::{ExitCode, Report};

/// Exit Refused, outcome "refused", owner the folder's HEAD, next checking out the request's branch and repeating,
/// under `verb`, with `lines` as the caller gives them.
pub(crate) fn head_refusal(verb: &str, lines: Vec<String>) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some("the folder's HEAD".to_string()),
        next: "check out the request's branch in the folder, then repeat the same command".to_string(),
        fields: Vec::new(),
    }
}
