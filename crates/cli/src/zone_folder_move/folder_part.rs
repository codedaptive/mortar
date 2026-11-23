//! folder_part: which of FOLDER_PARTS a zone folder's outcome needs in next.

use super::{FolderRefusal, ZoneFolderOutcome};

/// The index in FOLDER_PARTS of the part a folder's outcome needs; None when it needs none.
pub fn folder_part(outcome: &ZoneFolderOutcome) -> Option<usize> {
    match outcome {
        ZoneFolderOutcome::NoFolder | ZoneFolderOutcome::AtTip { .. } | ZoneFolderOutcome::Moved { .. } => None,
        ZoneFolderOutcome::Refused { why: FolderRefusal::InTheWay(_), .. } => Some(0),
        ZoneFolderOutcome::Refused { why: FolderRefusal::Unmerged(_), .. } => Some(1),
        ZoneFolderOutcome::Refused { why: FolderRefusal::NoZoneCommit, .. } => Some(2),
        ZoneFolderOutcome::Failed { .. } => Some(3),
    }
}
