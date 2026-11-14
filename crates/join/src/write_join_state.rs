//! write_join_state: writes mortar-join.json into a folder's git dir, whole or not at all.
//! No unit test: it writes and renames real files; encode_join_state_tests covers the text. Nagatha's integration
//! tests are to cover: the file holds exactly the text given; writing again replaces it; no
//! `mortar-join.json.tmp` is left beside it; a git dir that cannot be written to is refused as StateWrite naming
//! the temp file.

use crate::join_state_path::join_state_path;
use crate::JoinRunError;
use std::path::{Path, PathBuf};

/// Writes `text` to `mortar-join.json.tmp` in `git_dir`, then renames it over `mortar-join.json`, so a reader
/// finds the old state or the new one, never a torn one. Returns the path of `mortar-join.json`.
pub(crate) fn write_join_state(git_dir: &Path, text: &str) -> Result<PathBuf, JoinRunError> {
    let path = join_state_path(git_dir);
    let temp = git_dir.join("mortar-join.json.tmp");
    std::fs::write(&temp, text).map_err(|source| JoinRunError::StateWrite { path: temp.clone(), source })?;
    std::fs::rename(&temp, &path).map_err(|source| JoinRunError::StateWrite { path: path.clone(), source })?;
    Ok(path)
}
