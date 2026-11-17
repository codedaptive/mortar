//! The report of a `mortar sweep` whose refs were deleted and whose build dir sweep then failed. It never says
//! nothing half-done is left: the refs are gone, so a repeat sweeps only the build dirs.

use folders::FolderError;

use crate::folder::folder_error_why;
use crate::report::{ExitCode, OutValue, Report};

/// Exit 5, outcome "failed": a first line that the refs were deleted and clearing the idle build dirs failed, one
/// line per deleted ref, then why: folder_error_why's lines, or for Lock, Git, Oplog and DiskProbe (which it leaves
/// empty) the cli's own words, never another crate's Display. Next: repeat the same command. Field: deleted.
pub fn sweep_build_failure_report(deleted: &[String], error: &FolderError) -> Report {
    let mut lines = vec!["the refs listed were deleted; clearing the idle build dirs failed".to_string()];
    lines.extend(deleted.iter().map(|refname| format!("deleted {refname}")));
    match error {
        FolderError::Lock(_) => lines.push("a lock could not be taken".to_string()),
        FolderError::Git(_) => lines.push("a git run failed".to_string()),
        FolderError::Oplog(_) => lines.push("the op log could not be read or written".to_string()),
        FolderError::DiskProbe { path, reason } => {
            lines.push(format!("the free disk at {} could not be read: {reason}", path.display()))
        }
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
        | FolderError::RemovedRefsLeft { .. }
        | FolderError::ExtraRefRefused { .. } => lines.extend(folder_error_why(error)),
    }
    Report {
        verb: "sweep".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines,
        owner: None,
        next: "repeat the same command".to_string(),
        fields: vec![("deleted".to_string(), OutValue::Texts(deleted.to_vec()))],
    }
}
