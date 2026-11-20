//! The refusal remove answers for each step but Go.

use std::path::Path;

use super::RemoveStep;
use crate::make::FolderError;

/// None for Go. ZoneFolder names `zone`; OffBranch is NotAFolder naming `head` and `expected`; NotJoined names
/// `head` and `zone_ref`. All three name `folder`.
pub(crate) fn step_refusal(
    step: RemoveStep,
    folder: &Path,
    zone: &str,
    head: &str,
    expected: &str,
    zone_ref: &str,
) -> Option<FolderError> {
    let path = folder.to_path_buf();
    match step {
        RemoveStep::Go => None,
        RemoveStep::ZoneFolder => Some(FolderError::ZoneFolder {
            path,
            zone: zone.to_string(),
        }),
        RemoveStep::OffBranch => Some(FolderError::NotAFolder {
            path,
            reason: format!("its HEAD is on {head}, not its request branch {expected}"),
        }),
        RemoveStep::NotJoined => Some(FolderError::NotJoined {
            path,
            branch: head.to_string(),
            zone_branch: zone_ref.to_string(),
        }),
    }
}
