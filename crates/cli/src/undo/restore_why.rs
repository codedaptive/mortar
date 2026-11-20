//! restore_why: the why lines of a refused restore of a paused join, for `mr undo` of a resolve step.
//! Built from the error's fields, never join's Display; the match lists every variant with no wildcard.

use ::join::JoinRunError;

/// The lines saying why join's check_restore_join_state or restore_join_state refused. The errors a restore never
/// gives share one line saying so.
pub fn restore_why(error: &JoinRunError) -> Vec<String> {
    match error {
        JoinRunError::NotAResolveOp { op_id } => {
            vec![format!("op {op_id} records no paused join that undo can put back")]
        }
        JoinRunError::RecordedStateCorrupt { op_id, reason } => {
            vec![format!("the paused join op {op_id} recorded cannot be read back: {reason}")]
        }
        JoinRunError::StatePresent { state } => {
            vec![format!("{} holds another paused join, which undo never overwrites", state.display())]
        }
        JoinRunError::StateCorrupt { path, reason } => vec![
            format!("{} is not a paused-join file mortar wrote: {reason}", path.display()),
            "the file is kept".to_string(),
        ],
        JoinRunError::StateRead { path, source } => vec![format!("{} could not be read: {source}", path.display())],
        JoinRunError::StateWrite { path, source } => {
            vec![format!("{} could not be written: {source}", path.display())]
        }
        JoinRunError::StatePath(path) => vec![format!(
            "the conflicted path {} is not UTF-8, so the paused-join file cannot hold it",
            path.display()
        )],
        JoinRunError::RestoreStale { request_ref, expected, actual } => {
            let at = actual.as_ref().map_or_else(|| "nothing".to_string(), ToString::to_string);
            vec![format!("{request_ref} is at {at}, not at {expected}, where the paused join stopped")]
        }
        JoinRunError::Git(error) => vec![format!("git failed: {error}")],
        JoinRunError::Replay(_)
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
        | JoinRunError::ConflictedPathRefused { .. } => {
            vec!["join answered an error it never gives for a restore of a paused join".to_string()]
        }
    }
}
