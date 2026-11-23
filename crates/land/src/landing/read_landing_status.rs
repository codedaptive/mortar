//! read_landing_status: the landing in flight and whether its publish push intent is open, for status.
//! No unit test: it reads a file, a ref and the op log (S-5). Covered by publish_zit's
//! status::killed_inside_the_push_the_landing_status_names_the_open_intent and
//! status::with_no_publish_in_progress_the_landing_status_carries_no_intent.

use crate::landing::{open_push_intent_of, read_push_intent, read_resolved_landing_state, LandingStatus, PublishError};
use crate::squash::LandRefs;
use gitio::Git;

/// No landing state is None. Otherwise the state as read_resolved_landing_state resolves it, with `open_push` the
/// open push intent read_push_intent finds for its move. Read only: it takes no lock, writes nothing and never
/// contacts the remote, so the caller needs no lock. Finding the common dir failing is Git; reading the state,
/// integration's tip or the op log fails as State, Squash or Oplog.
pub fn read_landing_status(git: &Git, refs: &LandRefs) -> Result<Option<LandingStatus>, PublishError> {
    let common = git.common_dir().map_err(PublishError::Git)?;
    let Some(state) = read_resolved_landing_state(git, &common, refs)? else {
        return Ok(None);
    };
    let open_push = open_push_intent_of(read_push_intent(&common, refs, &state)?);
    Ok(Some(LandingStatus {
        change_id: state.change_id,
        zone: state.zone,
        squash: state.squash,
        origin_at_start: state.origin_at_start,
        integration: state.integration,
        open_push,
    }))
}
