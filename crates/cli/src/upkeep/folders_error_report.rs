//! The report of a `mortar disk` or `mortar sweep` that folders refused or failed. folders' own Display text, and
//! gitio's and oplog's inside it, is never printed (S-128 (4)): the why is the cli's words or folder_error_why's facts.

use folders::FolderError;

use crate::folder::{folder_error_why, folder_refusal};
use crate::lock::lock_report;
use crate::report::{failure_report, ExitCode, Report};

/// Lock is lock_report(verb). Git, Oplog and DiskProbe are failures (exit 5) led by "mortar <verb>:", in the cli's
/// words (DiskProbe names its path and reason). ExtraRefRefused is exit 1 naming the --ref to drop from the same
/// command (S-112). Every other variant, named one by one so a new one breaks the build, is folder_error_why's lines
/// with folder_refusal's exit and next: exit 1 "refused", or for Io exit 5 "failed".
pub fn folders_error_report(verb: &str, error: &FolderError) -> Report {
    let failed = |what: String| failure_report(verb, &format!("mortar {verb}: {what}"));
    match error {
        FolderError::Lock(lock) => lock_report(verb, lock),
        FolderError::Git(_) => failed("a git run failed".to_string()),
        FolderError::Oplog(_) => failed("the op log could not be read or written".to_string()),
        FolderError::DiskProbe { path, reason } => {
            failed(format!("the free disk at {} could not be read: {reason}", path.display()))
        }
        FolderError::ExtraRefRefused { refname, .. } => why_report(
            verb,
            error,
            ExitCode::Refused,
            format!("mortar {verb} with the same words without --ref {refname}, with --actor <name>"),
        ),
        FolderError::Plan(_)
        | FolderError::Record { .. }
        | FolderError::Io { .. }
        | FolderError::BelowFloor(_)
        | FolderError::DifferentFolder { .. }
        | FolderError::NotAFolder { .. }
        | FolderError::BranchCheckedOut { .. }
        | FolderError::BranchExists { .. }
        | FolderError::ZoneBranchMissing { .. }
        | FolderError::RequestBranchMissing { .. }
        | FolderError::AttemptMissing { .. }
        | FolderError::AttemptsUnsaved { .. }
        | FolderError::AttemptsRemain { .. }
        | FolderError::OrphanAttempt { .. }
        | FolderError::OrphanWithoutSnapshot { .. }
        | FolderError::RequestMovedOn { .. }
        | FolderError::AttemptFolder { .. }
        | FolderError::NotSaved { .. }
        | FolderError::MoveRefused { .. }
        | FolderError::BranchElsewhere { .. }
        | FolderError::NotRemovedUnsaved { .. }
        | FolderError::NotJoined { .. }
        | FolderError::ZoneFolder { .. }
        | FolderError::RefMoved { .. }
        | FolderError::AttemptRefsLeft { .. }
        | FolderError::RemovedRefsLeft { .. } => match folder_refusal(error) {
            Some((exit, next)) => why_report(verb, error, exit, next),
            None => failed("folders refused it for a reason the cli has no words for".to_string()),
        },
    }
}

/// folder_error_why's lines under `verb` with `exit` and `next`; outcome "refused" for exit 1, else "failed".
fn why_report(verb: &str, error: &FolderError, exit: ExitCode, next: String) -> Report {
    let outcome = if exit == ExitCode::Refused { "refused" } else { "failed" };
    Report {
        verb: verb.to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: folder_error_why(error),
        owner: None,
        next,
        fields: Vec::new(),
    }
}
