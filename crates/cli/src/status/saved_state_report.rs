//! saved_state_report: the report of a saved_state read that failed, for a verb that reads a folder's saved state
//! (mr status, mr disk). saved_state_report_tests covers each arm.

use snapshot::SnapshotError;

use crate::report::{failure_report, Report};
use crate::save::save_error_report;

/// saved_state's error for `verb`, about request `request` in zone `zone`. saved_state returns only BadTarget, Git,
/// FileSize, UnbornHead, PathNotUtf8 and CapMismatch. Three get the cli's words here, since save_error_report's text
/// for them carries the inner error's own: Git is "git failed while building the folder's tree", FileSize "the size
/// of <path> could not be read", BadTarget "the snapshot ref of request <r> in zone <z> cannot be named"; each is
/// failure_report for `verb`. UnbornHead, PathNotUtf8 and CapMismatch keep save_error_report's cli text, with `verb`
/// in place of "save". save_error_report's Oplog, Trailers, ChangeId and IndexNotSet arms are save's, and saved_state
/// never returns them.
pub fn saved_state_report(error: &SnapshotError, request: &str, zone: &str, verb: &str) -> Report {
    match error {
        SnapshotError::Git(_) => failure_report(verb, "git failed while building the folder's tree"),
        SnapshotError::FileSize { path, .. } => {
            failure_report(verb, &format!("the size of {} could not be read", path.display()))
        }
        SnapshotError::BadTarget { .. } => {
            failure_report(verb, &format!("the snapshot ref of request {request} in zone {zone} cannot be named"))
        }
        _ => Report { verb: verb.to_string(), ..save_error_report(error) },
    }
}
