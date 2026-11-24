//! finish_publish: publish's clean-up once the local origin is at the published commit: each member's squash ref,
//! in landing order, and the landing state go.
//! No unit test: it reads and deletes refs through the op log and removes a file. Covered by the publish_zit
//! tests publishing::publish_moves_the_remote_then_the_local_origin_and_cleans_up (squash ref and landing.json
//! gone, a done publish-squash record), kills::killed_between_the_steps_recovers_and_the_repeat_publishes_once (the
//! repeat after the kill leaves landing.json gone), and for a batch of two
//! batch::publish_batch_pushes_once_and_deletes_both_squashes_in_order,
//! batch::resquashed_member_is_kept_and_other_member_is_deleted and
//! batch::repeat_after_first_squash_cleanup_deletes_the_second_once.

use crate::landing::{integration_op, landing_applied, remove_landing_state, LandingState, PublishError};
use crate::squash::squash_ref;
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};
use std::path::Path;

/// For each member in landing order, when `refs/mortar/squash/<zone>` of that member holds its squash, one
/// `publish-squash` op, with the member's change ID, deletes it (its new value is the all-zero id of the squash id's
/// length); a squash ref already gone or holding another commit (the zone squashed again during the landing) is left
/// alone, so a repeat after a kill deletes each remaining ref once. Then the landing state is removed.
pub(crate) fn finish_publish(git: &Git, common: &Path, state: &LandingState, actor: &str) -> Result<(), PublishError> {
    for member in state.members() {
        let refname = squash_ref(&member.zone);
        if git.read_ref(&refname).map_err(PublishError::Git)?.as_ref() != Some(&member.squash) {
            continue;
        }
        let zero = Oid::parse(&"0".repeat(member.squash.as_str().len())).map_err(PublishError::Git)?;
        let op = integration_op(
            "publish-squash",
            &refname,
            &member.squash,
            &zero,
            &member.change_id,
            new_op_id(),
            actor,
            unix_now(),
        );
        landing_applied(apply(git, common, &op)).map_err(PublishError::Land)?;
    }
    remove_landing_state(common).map_err(PublishError::State)
}
