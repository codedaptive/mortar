//! A folder move's gitio outcome as a folder answer.

use std::path::Path;

use gitio::FolderMoveOutcome;

use crate::make::FolderError;

/// Moved is Ok; Refused is MoveRefused naming `folder` and the paths git said were in the way.
pub(crate) fn moved_or_refused(folder: &Path, outcome: FolderMoveOutcome) -> Result<(), FolderError> {
    match outcome {
        FolderMoveOutcome::Moved => Ok(()),
        FolderMoveOutcome::Refused(paths) => Err(FolderError::MoveRefused {
            path: folder.to_path_buf(),
            paths,
        }),
    }
}
