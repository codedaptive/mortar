//! saved_state_report: the report of a saved_state read that failed, for a verb that reads a folder's saved state
//! (mr status, mr disk). saved_state_report_tests covers each arm.

use snapshot::SnapshotError;

use crate::report::{failure_report, head_refusal, repeat_head_refusal, Report};
use crate::save::save_error_report;

/// saved_state's error for `verb`, about request `request` in zone `zone`; `attempt_branch` is an attempt folder's
/// short try branch (try/<zone>/<request>/<k>), None for any other folder. saved_state returns only BadTarget, Git,
/// FileSize, UnbornHead, PathNotUtf8 and CapMismatch. Three get the cli's words here, since save_error_report's text
/// for them carries the inner error's own: Git is "git failed while building the folder's tree", FileSize "the size
/// of <path> could not be read", BadTarget "the snapshot ref of request <r> in zone <z> cannot be named"; each is
/// failure_report for `verb`. UnbornHead is the HEAD refusal for `verb` with the error's own line: nothing was to
/// be saved, so it carries no "no save commit was written" line. Its next (disk keeps only the lines), in an attempt
/// folder, is checking out the try branch there and repeating (head_refusal), since the request's branch is not the
/// attempt's; elsewhere it is
/// repeat_head_refusal's. PathNotUtf8 and CapMismatch are save_error_report's report with only its verb field set to
/// `verb`. save_error_report's Oplog, Trailers, ChangeId and IndexNotSet arms are save's, and saved_state never
/// returns them.
pub fn saved_state_report(
    error: &SnapshotError,
    request: &str,
    zone: &str,
    verb: &str,
    attempt_branch: Option<&str>,
) -> Report {
    match error {
        SnapshotError::Git(_) => failure_report(verb, "git failed while building the folder's tree"),
        SnapshotError::FileSize { path, .. } => {
            failure_report(verb, &format!("the size of {} could not be read", path.display()))
        }
        SnapshotError::BadTarget { .. } => {
            failure_report(verb, &format!("the snapshot ref of request {request} in zone {zone} cannot be named"))
        }
        SnapshotError::UnbornHead { .. } => match attempt_branch {
            Some(branch) => head_refusal(
                verb,
                vec![error.to_string()],
                &format!("check out the branch {branch} in the attempt's folder, then repeat the same command"),
            ),
            None => repeat_head_refusal(verb, vec![error.to_string()]),
        },
        _ => Report { verb: verb.to_string(), ..save_error_report(error) },
    }
}
