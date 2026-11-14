//! decide_resolve: the resolve step from the paused state, where its request ref is now, the marker read and the
//! caller's tree. decide_resolve_tests covers each rule, the precedence between them and when the markers are read.

use crate::join_state::JoinState;
use crate::resolve_step::ResolveStep;
use crate::MarkedFile;
use gitio::Oid;

/// The rules, in this order: no state -> NothingToResolve; a saved `rebased_to` the request ref is already at ->
/// Finish (the op moved it; only the state is left); the request ref anywhere but the state's request tip -> Stale;
/// a saved `rebased_to` -> Rebase from the request tip to it; files not yet written -> WriteFiles with the state's
/// tree and files; any file with marker lines -> StillMarked with only those, in order; else Continue with the
/// caller's `tree`. `read_marked` is called only when the rules reach the marker check, so a file that cannot be read
/// never stands in the way of an earlier step; its error is returned as it is.
pub(crate) fn decide_resolve<E>(
    state: Option<&JoinState>,
    request_now: Option<&Oid>,
    read_marked: impl FnOnce() -> Result<Vec<MarkedFile>, E>,
    tree: &Oid,
) -> Result<ResolveStep, E> {
    let Some(state) = state else {
        return Ok(ResolveStep::NothingToResolve);
    };
    let from = state.request_tip.clone();
    if let Some(to) = state.rebased_to.as_ref().filter(|to| request_now == Some(*to)) {
        return Ok(ResolveStep::Finish { from, to: to.clone() });
    }
    if request_now != Some(&state.request_tip) {
        return Ok(ResolveStep::Stale { expected: from, actual: request_now.cloned() });
    }
    if let Some(to) = &state.rebased_to {
        return Ok(ResolveStep::Rebase { from, to: to.clone() });
    }
    if !state.files_written {
        return Ok(ResolveStep::WriteFiles { tree: state.tree.clone(), files: state.files.clone() });
    }
    let still: Vec<MarkedFile> = read_marked()?.into_iter().filter(|file| !file.lines.is_empty()).collect();
    if still.is_empty() {
        Ok(ResolveStep::Continue { tree: tree.clone() })
    } else {
        Ok(ResolveStep::StillMarked(still))
    }
}
