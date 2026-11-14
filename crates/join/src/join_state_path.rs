//! join_state_path: where a folder's paused-join state lives, mortar-join.json in its git dir. The one place the
//! state file's path is built, so resolve, write_join_state and paused_join agree on it (WO-join-6).

use std::path::{Path, PathBuf};

/// The path of mortar-join.json in `git_dir`. Touches no file.
pub(crate) fn join_state_path(git_dir: &Path) -> PathBuf {
    git_dir.join("mortar-join.json")
}
