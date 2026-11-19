//! The report of an `mr folder` sub-verb that folders refused or failed. folders' own Display text is never printed
//! for a folders variant: it names steps that are not cli commands; the why is folder_error_why's facts and the
//! exit and next are folder_refusal's.
//! No sibling test: every arm is pure, a FolderError value (its Lock, Git and Oplog errors are plain data a test
//! can build) mapped to a Report with no I/O, so a sibling unit test can cover each arm; folder_refusal_tests and folder_error_why_tests
//! cover the decisions it joins, and the integration tests crates/cli/tests/folder_runs/run_folder_{zone,request,remove,keep,drop}_tests.rs its
//! refusals (ZoneBranchMissing, NotRemovedUnsaved, AttemptsRemain, ZoneFolder, AttemptsUnsaved) through the binary.

use folders::FolderError;

use super::{folder_error_why, folder_refusal};
use crate::lock::lock_report;
use crate::report::{failure_report, ExitCode, Report};

const VERB: &str = "folder";

/// Lock is lock_report("folder"). Git and Oplog are failure_report with the inner error's own text (gitio's and
/// oplog's, not folders'), DiskProbe with its path and reason, Io with folder_error_why's line, each led by
/// "mr folder <sub>". Every other variant is exit 1, outcome "refused", folder_error_why's lines and
/// folder_refusal's next.
pub fn folder_error_report(sub: &str, error: &FolderError) -> Report {
    let failed = |what: String| failure_report(VERB, &format!("mr folder {sub}: {what}"));
    match error {
        FolderError::Lock(lock) => lock_report(VERB, lock),
        FolderError::Git(git) => failed(git.to_string()),
        FolderError::Oplog(oplog) => failed(oplog.to_string()),
        FolderError::DiskProbe { path, reason } => {
            failed(format!("the free disk at {} could not be read: {reason}", path.display()))
        }
        other => match folder_refusal(other) {
            Some((ExitCode::Refused, next)) => Report {
                verb: VERB.to_string(),
                exit: ExitCode::Refused,
                outcome: "refused".to_string(),
                lines: folder_error_why(other),
                owner: None,
                next,
                fields: Vec::new(),
            },
            // Io, the one variant folder_refusal answers with Failure.
            _ => failed(folder_error_why(other).join("; ")),
        },
    }
}
