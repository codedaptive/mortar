//! file_markers: the conflict-marker lines at one conflicted path in a resolve's next run. file_markers_tests covers
//! each rule; marked_lines_tests the marker rule itself.

use crate::marked_lines::marked_lines;
use std::io::ErrorKind;

/// A folder has no marker lines and `read` is not called: a folder has no file text, and a gitlink's checkout
/// is the usual one; the caller's tree decides, tree_with_paths_typed taking a gitlink entry from it or refusing a
/// folder entry as ConflictedPathIsFolder. Otherwise the marker lines of what `read` gives; a file that is gone has
/// none, its deletion being the resolution. Any other read error is returned.
pub(crate) fn file_markers(
    is_folder: bool,
    read: impl FnOnce() -> std::io::Result<Vec<u8>>,
) -> std::io::Result<Vec<usize>> {
    if is_folder {
        return Ok(Vec::new());
    }
    match read() {
        Ok(bytes) => Ok(marked_lines(&bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}
