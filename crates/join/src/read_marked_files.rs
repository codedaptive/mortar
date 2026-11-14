//! read_marked_files: reads each conflicted file in a folder and finds its conflict-marker lines.
//! No unit test: it reads real files (S-5); marked_lines_tests covers the marker rule. Nagatha's integration tests are
//! to cover, in a throwaway repository: a next run with a marker left is refused, naming each file and line still
//! marked (WO-join-3 Goal 3: "refuse while any conflict marker remains in those files, naming each file (and line)
//! still marked").

use crate::marked_lines::marked_lines;
use crate::JoinRunError;
use crate::MarkedFile;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// For each of `files` (relative to `folder`), in order, the file's marker lines. A file that is gone has none: its
/// deletion is the resolution. Any other read failure is FileRead naming the file.
pub(crate) fn read_marked_files(folder: &Path, files: &[PathBuf]) -> Result<Vec<MarkedFile>, JoinRunError> {
    files
        .iter()
        .map(|file| {
            let path = folder.join(file);
            let lines = match std::fs::read(&path) {
                Ok(bytes) => marked_lines(&bytes),
                Err(source) if source.kind() == ErrorKind::NotFound => Vec::new(),
                Err(source) => return Err(JoinRunError::FileRead { path, source }),
            };
            Ok(MarkedFile { path: file.clone(), lines })
        })
        .collect()
}
