//! read_resolved_landing_state: the landing state under `common` with a pending commit_files commit folded in or
//! dropped, as publish and the landing status read it.
//! No unit test: it reads a file and a ref (S-5); resolve_landing_state_tests covers the resolving. Covered by the
//! publish_zit tests through publish, the commit_files_zit tests
//! committing::named_commit_is_published_without_the_unnamed_edit and
//! kills::killed_after_update_ref_recovers_done_repeats_same_and_publishes (a pending commit), and publish_zit's
//! status::killed_inside_the_push_the_landing_status_names_the_open_intent and
//! status::with_no_publish_in_progress_the_landing_status_carries_no_intent.

use crate::landing::{read_landing_state, resolve_landing_state, LandingState, PublishError};
use crate::squash::{read_tip, LandRefs};
use gitio::Git;
use std::path::Path;

/// No landing state is None. A state with a pending commit is resolved (resolve_landing_state) against
/// integration's tip, `refs/heads/<integration_branch>`; one without is returned as read, no ref read. Reading the
/// state failing is State; reading integration's tip failing is Squash.
pub(crate) fn read_resolved_landing_state(
    git: &Git,
    common: &Path,
    refs: &LandRefs,
) -> Result<Option<LandingState>, PublishError> {
    let Some(state) = read_landing_state(common).map_err(PublishError::State)? else {
        return Ok(None);
    };
    if state.pending.is_none() {
        return Ok(Some(state));
    }
    let integration_ref = format!("refs/heads/{}", refs.integration_branch);
    Ok(Some(resolve_landing_state(state, &read_tip(git, &integration_ref).map_err(PublishError::Squash)?)))
}
