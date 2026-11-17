//! need_error_report: the report of an `mr need` that folders refused or that failed.
//! need_error_report_tests covers the Plan and NotAFolder arms. The Lock and fallback arms run lock_report,
//! failure_report and FolderError's Display, so they are RITs (S-101): Nagatha's integration group is to cover, with
//! the mortar binary as a child process in a request folder made by the landing verbs in a throwaway repository
//! (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - the folders lock held by a live pid while need runs is exit 3 naming the holder, and the record is unchanged;
//! - a git failure inside folders::need (its sparse-checkout add refused) is exit 5, next the same command again.

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
