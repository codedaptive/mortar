//! need_with_next: a finished need's report with the next step status gives, or with "mr status" when that step
//! could not be read (spec 02).

use crate::report::Report;
use crate::status::{status_next, StatusState};

/// `need` with next set from `read`: status_next of the state read, everything else unchanged. need has already
/// written by the time the next step is read, so a failed read never becomes need's result: `need` keeps its exit,
/// outcome, owner, fields and lines, gains the line `need is done but its next step could not be read: <reason>`
/// (the failure's lines joined with "; ", or its outcome when it has none) and gives next "mr status".
pub fn need_with_next(need: Report, read: Result<StatusState, Report>) -> Report {
    match read {
        Ok(state) => Report { next: status_next(&state), ..need },
        Err(failure) => {
            let reason = if failure.lines.is_empty() { failure.outcome } else { failure.lines.join("; ") };
            let mut lines = need.lines;
            lines.push(format!("need is done but its next step could not be read: {reason}"));
            Report { lines, next: "mr status".to_string(), ..need }
        }
    }
}
