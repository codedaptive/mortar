//! need_error_report: the report of an `mr need` that folders refused or that failed.
//! need_error_report_tests covers the Plan and NotAFolder arms. The Lock and fallback arms are pure too: lock_report,
//! failure_report and FolderError's Display map a FolderError value (plain data a test can build) to a Report with
//! no I/O, so a ZUT can cover them as well.

use folders::FolderError;

use crate::lock::lock_report;
use crate::report::{failure_report, ExitCode, Report};

/// Plan (a path the plan refuses: empty, absolute, holding NUL or a line break, or with a ".", ".." or empty
/// component) is exit 1 with the plan's reason, next a path inside the repository. NotAFolder (the folder has no
/// Mortar record) is exit 1 naming the folder and why, next making the request folder again. Lock (the folders lock
/// folders::need takes is held or cannot be taken) is lock_report. Every other error is failure_report.
pub fn need_error_report(error: &FolderError) -> Report {
    match error {
        FolderError::Plan(plan) => refused(plan.reason.clone(), "mr need <a path inside the repository>"),
        FolderError::NotAFolder { path, reason } => refused(
            format!("{} is not a Mortar folder: {reason}", path.display()),
            "make the request folder again, then mr need",
        ),
        FolderError::Lock(lock) => lock_report("need", lock),
        other => failure_report("need", &other.to_string()),
    }
}

fn refused(line: String, next: &str) -> Report {
    Report {
        verb: "need".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line],
        owner: Some("this request".to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}
