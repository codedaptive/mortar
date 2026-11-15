//! read_replayed_counterpart: the zone commit an earlier after_publish replayed a squash's source to.
//! No unit test: it runs git (S-5); replayed_counterpart_tests covers which commit it picks. Covered by
//! tests/after_publish_zit (replayed_source.rs):
//! the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin and
//! a_request_joined_after_the_replay_is_replayed_onto_origin.

use crate::after_publish::{replayed_counterpart, AfterPublishError};
use gitio::{Git, Oid};

/// Reads `squash_of` and each zone commit between `base` and `tip` (oldest first), then answers replayed_counterpart
/// of them: the first zone commit carrying the source's message and author, or None. A failed git run is Git.
pub(crate) fn read_replayed_counterpart(
    git: &Git,
    squash_of: &Oid,
    base: &Oid,
    tip: &Oid,
) -> Result<Option<Oid>, AfterPublishError> {
    let source = git.commit_info(squash_of).map_err(AfterPublishError::Git)?;
    let zone_commits = git
        .rev_list(base, tip)
        .map_err(AfterPublishError::Git)?
        .into_iter()
        .map(|commit| git.commit_info(&commit).map(|info| (commit, info)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(AfterPublishError::Git)?;
    Ok(replayed_counterpart(&source, &zone_commits))
}
