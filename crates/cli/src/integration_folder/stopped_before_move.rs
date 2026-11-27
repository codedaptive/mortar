//! stopped_before_move: the report of a read that failed after a verb moved integration and before the integration
//! folder moved, in move_integration_folder's refusal shape. land and back-out call it.

use crate::report::{ExitCode, Report};

/// Exit 1 "folder-not-moved": `done` (what the verb did to integration) first, then `error`'s lines joined "; " and
/// that the integration folder was not moved, then that `error`'s verb stopped. `error`'s verb, owner and fields
/// stay; its next (each ends "repeat the same command") ends with `repeat`, which moves the folder.
pub(crate) fn stopped_before_move(done: &str, error: Report, repeat: &str) -> Report {
    Report {
        exit: ExitCode::Refused,
        outcome: "folder-not-moved".to_string(),
        lines: vec![
            done.to_string(),
            format!("{}, so the integration folder was not moved", error.lines.join("; ")),
            format!("{} stopped here", error.verb),
        ],
        next: format!("{}: {repeat}", error.next),
        ..error
    }
}
