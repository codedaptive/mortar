//! finish_resolve: a resolve's final step: the one op that moves the request ref, then the state's removal.
//! No unit test: it applies an op and removes a real file; rebase_op_tests, op_applied_tests,
//! folder_tree_values_tests and join_state_value_tests cover its parts. resolve_on_base.rs's header lists the
//! integration cases.

use crate::folder_tree_values::folder_tree_values;
use crate::join_state::JoinState;
use crate::join_state_value::join_state_value;
use crate::op_applied::op_applied;
use crate::rebase_op::rebase_op;
use crate::remove_join_state::remove_join_state;
use crate::{JoinAnswer, JoinRunError};
use gitio::{Git, Oid};
use oplog::{apply_with_values, new_op_id, unix_now};
use std::path::Path;

/// Moves the request ref from `from` to `to` by one op, as join's rebase does, recording in its values through
/// apply_with_values `folder_tree` under FOLDER_TREE and then the state as mortar-join.json held it before the
/// final run under JOIN_STATE, then removes the state at `path` and answers Rebased with that folder_tree.
/// With no folder_tree it refuses as StateCorrupt, and with a conflicted path that is not UTF-8 as StatePath; both
/// refusals come before the op, so the state is kept and nothing moves.
pub(crate) fn finish_resolve(
    git: &Git,
    path: &Path,
    state: &JoinState,
    from: Oid,
    to: Oid,
    folder_tree: Option<Oid>,
    actor: &str,
) -> Result<JoinAnswer, JoinRunError> {
    let mut values = folder_tree_values(folder_tree.as_ref())
        .map_err(|reason| JoinRunError::StateCorrupt { path: path.to_path_buf(), reason })?;
    values.push(join_state_value(state).map_err(JoinRunError::StatePath)?);
    let common = git.common_dir().map_err(JoinRunError::Git)?;
    let op = rebase_op(&state.request_ref, &from, &to, &state.onto, new_op_id(), actor, unix_now());
    op_applied(apply_with_values(git, &common, &op, &values).map(|valued| valued.outcome))?;
    remove_join_state(path)?;
    Ok(JoinAnswer::Rebased { request_ref: state.request_ref.clone(), from, to, folder_tree })
}
