//! resolved_folder_tree: the tree the caller moves the folder from after a resolve completes the replay.
//! resolved_folder_tree_tests covers the calls made, an earlier round's path taken, and a refused path.

use crate::join_state::JoinState;
use crate::replay_git::ReplayGit;
use crate::tree_refusal::tree_refusal;
use crate::JoinRunError;
use gitio::Oid;

/// The old request tip's tree with every path resolved in this join, all rounds (`state.all_files`), taken from
/// `caller_tree`, the folder's tree. Its other paths stay as the old tip has them, so the folder's edits outside the
/// resolved paths remain local edits when it moves. A failure reading the old tip is Git; a failure
/// building the tree is mapped by tree_refusal against all_files. Writes a tree object; moves no ref.
pub(crate) fn resolved_folder_tree(
    git: &impl ReplayGit,
    state: &JoinState,
    caller_tree: &Oid,
) -> Result<Oid, JoinRunError> {
    let info = git.commit_info(&state.request_tip).map_err(JoinRunError::Git)?;
    git.tree_with_paths_typed(&info.tree, caller_tree, &state.all_files)
        .map_err(|error| tree_refusal(error, &state.all_files))
}
