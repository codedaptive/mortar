//! join_state_value: the value under which a resolve's final join-rebase op records the paused state.
//! join_state_value_tests covers the name, a hand-written text, the clearing of rebased_to and folder_tree, the round
//! trip through decode_join_state and the refusal of a non-UTF-8 path.

use crate::encode_join_state::encode_join_state;
use crate::join_state::JoinState;
use crate::join_state_name::JOIN_STATE;
use std::path::PathBuf;

/// Answers JOIN_STATE and the text of `state` as encode_join_state writes it, with rebased_to and folder_tree set to
/// null and no trailing newline. Why clear those two: resolve sets them only after reading the state, just before
/// saving it ahead of the final op (resolve_on_base.rs, the AfterReplayStep::Rebase arm), and decide_resolve
/// answers Continue only for a state whose rebased_to is None, so the cleared text is exactly what mortar-join.json
/// held before the final resolve run, on a repeat of that run too. A conflicted path that is not UTF-8 is returned
/// as the error.
pub(crate) fn join_state_value(state: &JoinState) -> Result<(String, String), PathBuf> {
    let paused = JoinState { rebased_to: None, folder_tree: None, ..state.clone() };
    let text = encode_join_state(&paused)?;
    Ok((JOIN_STATE.to_string(), text.strip_suffix('\n').unwrap_or(&text).to_string()))
}
