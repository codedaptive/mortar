//! read_squash_of: the zone's squash on a first-parent line and the zone tip it names.
//! No unit test: it runs git (S-5); squash_of_in_tests covers what it picks from each commit's trailers. Covered by
//! tests/after_publish_zit (zone_moves.rs): the_publishing_zone_with_no_later_joins_is_put_on_origin and
//! the_publishing_zone_with_a_later_join_is_replayed_onto_origin read the squash land's own squash wrote, and
//! (merge_landing.rs) the_publishing_zone_landed_as_a_merge_is_found_through_the_second_parent reads it through a
//! landing merge's second parent, and (replayed_source.rs)
//! the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin and
//! a_request_joined_after_the_replay_is_replayed_onto_origin, and (replay_provenance.rs) its three cases, read it
//! from a squash whose named tip the zone holds only as a recorded replay, or not at all; (older_squash.rs) both
//! cases read the earlier-published squash from the line behind the publishing one (rule O, through
//! read_publishing).

use crate::after_publish::{squash_of_in, AfterPublishError};
use gitio::{Git, Oid};

/// Walks the first-parent line from `origin` newest first, back to what `zone_tip` reaches; at each commit, the
/// commit itself and then its other parents, because a landing merge (integration, squash) carries no trailers and
/// holds the squash as its second parent. The first commit whose trailers carry Mortar-Zone `zone` and a
/// Mortar-Squash-Of gives (that commit, the Mortar-Squash-Of value as an id); None when no commit does. A failed
/// git run or a Mortar-Squash-Of that is not an id is Git.
pub(crate) fn read_squash_of(
    git: &Git,
    origin: &Oid,
    zone_tip: &Oid,
    zone: &str,
) -> Result<Option<(Oid, Oid)>, AfterPublishError> {
    let squash_of = |commit: &Oid, message: &str| -> Result<Option<(Oid, Oid)>, AfterPublishError> {
        let trailers = git.trailers(message).map_err(AfterPublishError::Git)?;
        let named = squash_of_in(&trailers, zone).map(Oid::parse).transpose().map_err(AfterPublishError::Git)?;
        Ok(named.map(|named| (commit.clone(), named)))
    };
    for commit in git.first_parent_list(zone_tip, origin).map_err(AfterPublishError::Git)?.iter().rev() {
        let info = git.commit_info(commit).map_err(AfterPublishError::Git)?;
        if let Some(found) = squash_of(commit, &info.message)? {
            return Ok(Some(found));
        }
        for parent in info.parents.iter().skip(1) {
            if let Some(found) = squash_of(parent, &git.commit_info(parent).map_err(AfterPublishError::Git)?.message)? {
                return Ok(Some(found));
            }
        }
    }
    Ok(None)
}
