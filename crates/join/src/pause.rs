//! pause: stops a conflicted replay, saving its state for `mr resolve`.
//! No unit test: it runs git and writes real files; after_replay_tests covers the state it is given. Integration
//! tests are to cover, in a throwaway repository: a conflict lists the files, writes mortar-join.json and moves no
//! ref (join.rs's header); a resolve whose continued replay conflicts again writes a new state for the new commit,
//! moves no ref and lists the files.

use crate::join_state::JoinState;
use crate::save_join_state::save_join_state;
use crate::{JoinAnswer, JoinRunError};
use gitio::Git;
use std::path::Path;

/// Writes `state` as mortar-join.json in `folder`'s git dir and answers Conflicted naming it. A conflicted path
/// that is not UTF-8 is refused (StatePath) before anything is written.
pub(crate) fn pause(git: &Git, folder: &Path, state: JoinState) -> Result<JoinAnswer, JoinRunError> {
    let git_dir = git.git_dir(folder).map_err(JoinRunError::Git)?;
    let path = save_join_state(&git_dir, &state)?;
    Ok(JoinAnswer::Conflicted { commit: state.commit, files: state.files, state: path })
}
