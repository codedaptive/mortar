//! The exit code and next step of an `mr folder` that folders refused, by the error's variant alone. Its why
//! lines are folder_error_why's; folders' own Display text is never printed, as it names steps that are not cli
//! commands.

use folders::FolderError;

use crate::report::ExitCode;

/// None for Lock, Git, Oplog and DiskProbe: those are reported as every cli verb reports them (lock_report,
/// failure_report). Io is Failure (exit 5), its next a repeat of the same command. Every other
/// variant is Refused (exit 3) with one next step for the harness: a command that runs now as written (every harness
/// command it names carries `--actor <name>`), or the step in words when none does. No next is aimed at an
/// agent; placeholders such as `<k>` and `<name>` are printed as written.
pub fn folder_refusal(error: &FolderError) -> Option<(ExitCode, String)> {
    let next = match error {
        FolderError::Lock(_) | FolderError::Git(_) | FolderError::Oplog(_) | FolderError::DiskProbe { .. } => {
            return None
        }
        FolderError::Io { .. } => {
            return Some((ExitCode::Failure, "repeat the same command".to_string()))
        }
        FolderError::Plan(_) => "repeat with names and paths the folder plan takes",
        FolderError::Record { .. } => {
            "make the folder again at another path with mr folder zone <zone> <path> --actor <name> or mr \
             folder request <zone> <request> <path> --paths <path>... --actor <name>"
        }
        FolderError::BelowFloor(_) => "free disk space until it is over the floor, then repeat",
        FolderError::DifferentFolder { .. } => "choose another path for the folder, then repeat",
        FolderError::NotAFolder { .. } => {
            "choose another path, or move what is at that path out of the way, then repeat"
        }
        FolderError::BranchCheckedOut { .. } => {
            "use the folder that has the branch checked out, or remove it, then repeat"
        }
        FolderError::BranchExists { .. } => "choose another request name, then repeat",
        FolderError::ZoneBranchMissing { .. } => {
            "mr folder zone <zone> <path> --start <commit> --actor <name>, then repeat"
        }
        FolderError::RequestBranchMissing { .. } => {
            "mr folder request <zone> <request> <path> --paths <path>... --actor <name>, then repeat"
        }
        FolderError::AttemptMissing { .. } => {
            "mr folder keep <k> --actor <name> with an attempt that exists, or mr folder drop --actor <name>"
        }
        FolderError::AttemptsUnsaved { .. } => "repeat once each listed attempt's work is saved",
        FolderError::AttemptsRemain { .. } => {
            "mr folder keep <k> --actor <name> or mr folder drop --actor <name>, run in the request's folder"
        }
        // Only met when a try branch vanished after remake_orphans ran in this same command.
        FolderError::OrphanAttempt { .. } => "repeat the same command",
        FolderError::OrphanWithoutSnapshot { .. } => {
            "copy out anything wanted from the listed folders, remove each by hand, then repeat"
        }
        FolderError::RequestMovedOn { .. } => {
            "bring the request branch's commits into the attempt, or make the attempts again, then repeat"
        }
        FolderError::AttemptFolder { .. } => {
            "after keep or drop, run mr folder remove <path> --actor <name> with the request's own folder as <path>"
        }
        FolderError::NotSaved { .. } | FolderError::NotRemovedUnsaved { .. } => {
            "repeat once the folder's work is saved"
        }
        FolderError::MoveRefused { .. } => "move the local changes at the listed paths out of the way, then repeat",
        FolderError::BranchElsewhere { .. } => {
            "read who moved the branch with mr log --ref <full ref name>, then repeat"
        }
        FolderError::NotJoined { .. } => "repeat once the request has joined its zone",
        FolderError::ZoneFolder { .. } => {
            "leave the zone folder in place; mr folder remove <path> --actor <name> takes request folders only"
        }
        FolderError::RefMoved { .. } | FolderError::AttemptRefsLeft { .. } => "repeat the same command",
        FolderError::RemovedRefsLeft { .. } => "nothing to do now: the refs hold only work already in the zone",
        FolderError::ExtraRefRefused { .. } => "repeat without that ref",
    };
    Some((ExitCode::Refused, next.to_string()))
}
