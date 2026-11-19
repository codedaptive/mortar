//! finish_publish: publish's clean-up once the local origin is at the published commit: the zone's squash ref and
//! the landing state go.
//! No unit test: it reads and deletes a ref through the op log and removes a file. Covered by the publish_zit
//! tests publish_moves_the_remote_then_the_local_origin_and_cleans_up (squash ref and landing.json gone, a done
//! publish-squash record) and killed_between_the_steps_recovers_and_the_repeat_publishes_once (the repeat after the
//! kill leaves landing.json gone).

use crate::landing::{integration_op, landing_applied, remove_landing_state, LandingState, PublishError};
use crate::squash::squash_ref;
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};
use std::path::Path;

/// When `refs/mortar/squash/<zone>` holds the state's squash, one `publish-squash` op deletes it (its new value is
/// the all-zero id of the squash id's length); a squash ref already gone or holding another commit is left alone,
/// so a repeat after a kill finishes once. Then the landing state is removed.
pub(crate) fn finish_publish(git: &Git, common: &Path, state: &LandingState, actor: &str) -> Result<(), PublishError> {
    let refname = squash_ref(&state.zone);
    if git.read_ref(&refname).map_err(PublishError::Git)?.as_ref() == Some(&state.squash) {
        let zero = Oid::parse(&"0".repeat(state.squash.as_str().len())).map_err(PublishError::Git)?;
        let op = integration_op(
            "publish-squash",
            &refname,
            &state.squash,
            &zero,
            &state.change_id,
            new_op_id(),
            actor,
            unix_now(),
        );
        landing_applied(apply(git, common, &op)).map_err(PublishError::Land)?;
    }
    remove_landing_state(common).map_err(PublishError::State)
}
