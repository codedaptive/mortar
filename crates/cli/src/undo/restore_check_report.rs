//! restore_check_report: the report of `mr undo` when join's check_restore_join_state refused a restore of a paused
//! join, before anything moved (S-148, S-155). The match lists every variant with no wildcard (S-128 (4)).
//! restore_check_report_tests covers each refusal and failure, running restore_why, which is pure on those errors,
//! inside. The Git case, a_git_failure_is_a_failure_to_repeat, runs gitio's GitError Display, a RIT in
//! crates/cli/tests/cli_rit/restore_git_errors.rs (S-101).

use ::join::JoinRunError;

use super::restore_why;
use crate::report::{ExitCode, Report};

/// The why lines, then "nothing was changed". An op recording no usable paused join, another paused join present
/// and a state file mortar did not write are exit 1 "refused", each with its own next step. Git or the disk failing,
/// and every error the check never gives, are exit 5 "failed": a repeat is safe because nothing moved (S-71).
pub fn restore_check_report(error: &JoinRunError) -> Report {
    let refused = |next: &str| (ExitCode::Refused, "refused", next.to_string());
    let failed = (ExitCode::Failure, "failed", "repeat the same command".to_string());
    let (exit, outcome, next) = match error {
        JoinRunError::NotAResolveOp { .. } | JoinRunError::RecordedStateCorrupt { .. } => {
            refused("change the files, mr save, then mr join")
        }
        JoinRunError::StatePresent { .. } => refused("mr status"),
        JoinRunError::StateCorrupt { .. } => refused("mend or remove that file by hand, then mr undo"),
        JoinRunError::StateRead { .. }
        | JoinRunError::Git(_)
        | JoinRunError::RestoreStale { .. }
        | JoinRunError::StateWrite { .. }
        | JoinRunError::StatePath(_)
        | JoinRunError::Replay(_)
        | JoinRunError::Oplog(_)
        | JoinRunError::ZoneMissing(_)
        | JoinRunError::RequestMissing(_)
        | JoinRunError::Unrelated { .. }
        | JoinRunError::OpFailed(_)
        | JoinRunError::OpRepeated(_)
        | JoinRunError::NothingToResolve { .. }
        | JoinRunError::StillMarked(_)
        | JoinRunError::StateStale { .. }
        | JoinRunError::StateRemove { .. }
        | JoinRunError::FileRead { .. }
        | JoinRunError::ConflictedPathIsFolder { .. }
        | JoinRunError::ConflictedPathRefused { .. } => failed,
    };
    let mut lines = restore_why(error);
    lines.push("nothing was changed".to_string());
    Report {
        verb: "undo".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next,
        fields: Vec::new(),
    }
}
