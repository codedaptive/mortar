//! The report of a resolve that resolve_on_base refused.
//! resolve_refusal_report_tests covers no paused join, markers left and the refusals that carry join's own text
//! (through join_refusal_report and join's exported, pure Display, so a unit test may use it).

use ::join::JoinRunError;

use crate::join::join_refusal_report;
use crate::report::{with_moved_first, ExitCode, Report};

const VERB: &str = "resolve";
const REQUEST: &str = "this request";

/// `moved` is moved_before's line for what the steps before resolve moved. No paused join is exit 0
/// "nothing-to-resolve", next "mr join", with `moved` as its first line. Markers left are exit 2 "still-marked", each
/// file with its marker line numbers, next "fix the files, then mr resolve"; `moved` is named in its first line,
/// which then says no other commit or ref moved. A request
/// ref moved since the pause (StateStale), a conflicted path that is a folder or that gitio refused, and a state file
/// join did not write (StateCorrupt, which is kept) are exit 1 "refused", owned by this request, with `moved` as the
/// first line and then the error's text. Every other error is join_refusal_report's report, given `moved`, under
/// the verb resolve.
pub fn resolve_refusal_report(error: &JoinRunError, moved: Option<&str>) -> Report {
    let (lines, next) = match error {
        JoinRunError::NothingToResolve { .. } => {
            return Report {
                verb: VERB.to_string(),
                exit: ExitCode::Done,
                outcome: "nothing-to-resolve".to_string(),
                lines: with_moved_first(
                    moved,
                    vec!["no join is paused in this folder; nothing to resolve".to_string()],
                ),
                owner: None,
                next: "mr join".to_string(),
                fields: Vec::new(),
            };
        }
        JoinRunError::StillMarked(files) => {
            let mut lines = vec![match moved {
                None => "conflict markers remain in these files; no commit was made and no ref moved:".to_string(),
                Some(m) => format!(
                    "conflict markers remain in these files; {m}; no other commit was made and no other ref moved:"
                ),
            }];
            lines.extend(files.iter().map(|file| {
                let numbers: Vec<String> = file.lines.iter().map(ToString::to_string).collect();
                format!("  {} (lines {})", file.path.display(), numbers.join(", "))
            }));
            return Report {
                verb: VERB.to_string(),
                exit: ExitCode::Conflict,
                outcome: "still-marked".to_string(),
                lines,
                owner: None,
                next: "fix the files, then mr resolve".to_string(),
                fields: Vec::new(),
            };
        }
        JoinRunError::StateStale { request_ref, expected, .. } => {
            (vec![error.to_string()], format!("put {request_ref} back at {expected}, then mr resolve"))
        }
        JoinRunError::ConflictedPathIsFolder { .. } | JoinRunError::ConflictedPathRefused { .. } => {
            (vec![error.to_string()], "put a file at that path, then mr resolve".to_string())
        }
        JoinRunError::StateCorrupt { .. } => (
            vec![error.to_string(), "the file is kept".to_string()],
            "mend or remove that file by hand, then mr join".to_string(),
        ),
        _ => return Report { verb: VERB.to_string(), ..join_refusal_report(error, moved) },
    };
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: with_moved_first(moved, lines),
        owner: Some(REQUEST.to_string()),
        next,
        fields: Vec::new(),
    }
}
