//! fetch_origin: brings the remote's moved origin branch home: fetches it and fast-forwards the local origin onto it.
//! No unit test: it fetches from the remote and moves a ref through the op log. Covered by the publish_zit test
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again (local main at the other clone's commit).

use crate::landing::{integration_op, landing_applied, PublishError};
use crate::squash::{read_tip, LandRefs};
use gitio::{FetchRefOutcome, Git, Oid};
use oplog::{apply_protected, new_op_id, unix_now, ChangeId, ProtectedRef};
use std::path::Path;

/// Fetches the remote's `refs/heads/<origin_branch>` onto `refs/mortar/fetched/<origin_branch>` (RemoteMissing when
/// the remote has none) and reads it as F. Then one `publish-fetched` op, with the origin branch protected by
/// `publish`, moves the local origin from `local` to F, so it only ever moves forward: a fetched F that is not a
/// descendant of `local` is refused (Land). Returns F.
pub(crate) fn fetch_origin(
    git: &Git,
    common: &Path,
    refs: &LandRefs,
    local: &Oid,
    change_id: &ChangeId,
    actor: &str,
) -> Result<Oid, PublishError> {
    let origin_ref = format!("refs/heads/{}", refs.origin_branch);
    let fetched_ref = format!("refs/mortar/fetched/{}", refs.origin_branch);
    let outcome = git.fetch_ref(&refs.remote, &origin_ref, &fetched_ref).map_err(PublishError::Git)?;
    if outcome == FetchRefOutcome::MissingSource {
        return Err(PublishError::RemoteMissing { remote: refs.remote.clone(), refname: origin_ref });
    }
    let found = read_tip(git, &fetched_ref).map_err(PublishError::Squash)?;
    let op = integration_op("publish-fetched", &origin_ref, local, &found, change_id, new_op_id(), actor, unix_now());
    let protected = [ProtectedRef { refname: origin_ref, by: "publish".to_string() }];
    landing_applied(apply_protected(git, common, &op, &protected)).map_err(PublishError::Land)?;
    Ok(found)
}
