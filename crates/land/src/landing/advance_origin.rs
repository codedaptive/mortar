//! advance_origin: publish's step 2, moving the local origin branch to the published commit.
//! No unit test: it moves a ref through the op log. Covered by the publish_zit tests
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up (a done publish-origin record) and
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once (the repeat moves local main to the landed commit).

use crate::landing::{integration_op, landing_applied, LandingState, PublishError};
use crate::squash::LandRefs;
use gitio::Git;
use oplog::{apply_protected, new_op_id, unix_now, ProtectedRef};
use std::path::Path;

/// One `publish-origin` op, with the origin branch protected by `publish`, moves `refs/heads/<origin_branch>` from
/// the state's origin_at_start to its newest member's integration commit, with that member's change ID: a
/// fast-forward by construction, and anything else is refused (Land), as is an op that fails or was already done.
pub(crate) fn advance_origin(
    git: &Git,
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
    actor: &str,
) -> Result<(), PublishError> {
    let origin_ref = format!("refs/heads/{}", refs.origin_branch);
    let op = integration_op(
        "publish-origin",
        &origin_ref,
        &state.origin_at_start,
        &state.newest.integration,
        &state.newest.change_id,
        new_op_id(),
        actor,
        unix_now(),
    );
    let protected = [ProtectedRef { refname: origin_ref, by: "publish".to_string() }];
    landing_applied(apply_protected(git, common, &op, &protected)).map_err(PublishError::Land)
}
