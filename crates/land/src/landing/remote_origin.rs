//! remote_origin: where the remote's origin branch stands.
//! No unit test: it runs git ls-remote against the remote. Covered by the publish_zit tests
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up,
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again and
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once, which read the remote through it.

use crate::landing::PublishError;
use crate::squash::LandRefs;
use gitio::{Git, Oid};

/// The id of `refs/heads/<origin_branch>` on `refs.remote`, or None when the remote has no such ref. ls-remote
/// matches a pattern against refname tails, so only the entry named exactly that ref counts; a failed run is Git.
pub(crate) fn remote_origin(git: &Git, refs: &LandRefs) -> Result<Option<Oid>, PublishError> {
    let refname = format!("refs/heads/{}", refs.origin_branch);
    let listed = git.ls_remote(&refs.remote, std::slice::from_ref(&refname)).map_err(PublishError::Git)?;
    Ok(listed.into_iter().find(|(_, name)| *name == refname).map(|(oid, _)| oid))
}
