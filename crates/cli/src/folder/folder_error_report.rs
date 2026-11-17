//! The report of an `mr folder` sub-verb that folders refused or failed. folders' own Display text is never printed
//! for a folders variant: it names steps that are not cli commands (S-112); the why is folder_error_why's facts and the
//! exit and next are folder_refusal's.
//! No sibling test (S-104): a FolderError's Lock, Git and Oplog variants hold errors that only a real lock file, git
//! run or op log make, so the mapping is reached through the runners; folder_refusal_tests and folder_error_why_tests
//! cover the decisions it joins, and the RITs crates/cli/tests/cli_rit/run_folder_{zone,request,remove,keep,drop}_tests.rs its
//! refusals (ZoneBranchMissing, NotRemovedUnsaved, AttemptsRemain, ZoneFolder, AttemptsUnsaved) through the binary.
//! RITs left for Nagatha (cross-zone, listed only, not written): a held lock reported by lock_report; a git failure
//! inside folders reported as failure_report, exit 5.

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
