//! save_join_state: encodes a JoinState and writes it as mortar-join.json in a git dir.
//! No unit test: it writes real files; encode_join_state_tests covers the text. Integration tests are to cover, in
//! a throwaway repository: a conflict writes the state to mortar-join.json in the folder's git dir; resolve reads
//! that state back.

use crate::encode_join_state::encode_join_state;
use crate::join_state::JoinState;
use crate::write_join_state::write_join_state;
use crate::JoinRunError;
use std::path::{Path, PathBuf};

/// Encodes `state` and writes it whole into `git_dir`, returning the path of mortar-join.json. A conflicted path that
/// is not UTF-8 is refused (StatePath) before anything is written.
pub(crate) fn save_join_state(git_dir: &Path, state: &JoinState) -> Result<PathBuf, JoinRunError> {
    let text = encode_join_state(state).map_err(JoinRunError::StatePath)?;
    write_join_state(git_dir, &text)
}
