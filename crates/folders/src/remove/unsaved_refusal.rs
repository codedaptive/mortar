//! remove's first check: the folder's work is saved and no file of it was refused by the snapshot size cap.

use std::path::{Path, PathBuf};

use crate::make::FolderError;

/// NotRemovedUnsaved for `folder` when `saved` is false or `refused` names any file; None otherwise. The caller
/// passes snapshot's saved_state(..).saved as `saved` and the files it refused as `refused`; a refused list
/// refuses on its own as well, since those files are in no snapshot. `cap` is the size cap in bytes, carried into
/// the refusal's text.
pub(crate) fn unsaved_refusal(folder: &Path, saved: bool, refused: &[PathBuf], cap: Option<u64>) -> Option<FolderError> {
    if saved && refused.is_empty() {
        return None;
    }
    Some(FolderError::NotRemovedUnsaved {
        path: folder.to_path_buf(),
        refused: refused.to_vec(),
        cap,
    })
}
