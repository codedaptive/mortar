//! read_marked_files: reads each conflicted file in a folder and finds its conflict-marker lines.
//! No unit test: it reads real files (S-5); file_markers_tests covers which paths are read and what a missing file
//! or a folder has, marked_lines_tests the marker rule. Nagatha's integration tests are to cover, in a throwaway
//! repository: a next run with a marker left is refused, naming each file and line still marked (WO-join-3 Goal 3:
//! "refuse while any conflict marker remains in those files, naming each file (and line) still marked"); a gitlink
//! path (a folder in the folder) is not read (S-64).

use crate::file_markers::file_markers;
use crate::JoinRunError;
use crate::MarkedFile;
use std::path::{Path, PathBuf};

/// For each of `files` (relative to `folder`), in order, the file's marker lines. A folder has none and is not read:
/// a folder has no file text (a gitlink's checkout, S-64, is the usual one); a symlink is not a folder here and is
/// read, so one to a folder fails as FileRead. A file that is gone has none: its deletion is the resolution. Any other
/// read failure is FileRead naming the file.
pub(crate) fn read_marked_files(folder: &Path, files: &[PathBuf]) -> Result<Vec<MarkedFile>, JoinRunError> {
    files
        .iter()
        .map(|file| {
            let path = folder.join(file);
            let is_folder = std::fs::symlink_metadata(&path).is_ok_and(|meta| meta.is_dir());
            let lines = file_markers(is_folder, || std::fs::read(&path))
                .map_err(|source| JoinRunError::FileRead { path, source })?;
            Ok(MarkedFile { path: file.clone(), lines })
        })
        .collect()
}
