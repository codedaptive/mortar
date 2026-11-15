//! read_record_move: what a zone's replay calls for of its replay record: a move, none, or a commit it cannot name.
//! No unit test: it runs git (S-5); replayed_position_tests covers which commit it records. Covered by
//! tests/after_publish_zit (replayed_source.rs):
//! the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin and
//! a_request_joined_after_the_replay_is_replayed_onto_origin, (replay_provenance.rs) its three cases and
//! (record_consumed.rs) its three cases, each writing the record in after_publish(b, [land]); the two saves case and
//! the record_consumed cases read it back; (replay_drop.rs)
//! a_zone_whose_replay_dropped_a_commit_owing_a_record_stays_put_and_moves_at_its_own_publish (Unmappable).

use crate::after_publish::{
    replay_record_ref, replayed_position, squash_of_in, AfterPublishError, RecordMapping, RecordRead,
};
use crate::squash::squash_ref;
use gitio::{Git, GitError, Oid};
use oplog::RefMove;

/// When `refs/mortar/squash/<zone>` holds a squash whose trailers name a Mortar-Squash-Of for `zone`, the record
/// `replay_record_ref(zone, source)` follows the replay of `base..tip` onto `origin` that gave `new_tip`:
/// replayed_position, given the source and the record's value, looks the commit standing for the source up in the
/// old and new ranges; Mapped is Move, setting the record to its replay, and Unmappable is Unmappable naming the
/// source. NotOwed when there is no squash, no Mortar-Squash-Of, or replayed_position answers NotOwed. The source
/// commit itself is never read, so a pruned source is no error. A failed git run or a Mortar-Squash-Of that is not
/// an id is Git.
pub(crate) fn read_record_move(
    git: &Git,
    zone: &str,
    base: &Oid,
    tip: &Oid,
    origin: &Oid,
    new_tip: &Oid,
) -> Result<RecordRead, AfterPublishError> {
    let Some(squash) = git.read_ref(&squash_ref(zone)).map_err(AfterPublishError::Git)? else {
        return Ok(RecordRead::NotOwed);
    };
    let Some(source) = squash_of_in(
        &git.commit_info(&squash).and_then(|info| git.trailers(&info.message)).map_err(AfterPublishError::Git)?,
        zone,
    )
    .map(Oid::parse)
    .transpose()
    .map_err(AfterPublishError::Git)?
    else {
        return Ok(RecordRead::NotOwed);
    };
    let refname = replay_record_ref(zone, source.as_str());
    let existing = git.read_ref(&refname).map_err(AfterPublishError::Git)?;
    let list = |read: fn(&Git, &Oid, &Oid) -> Result<Vec<Oid>, GitError>, from: &Oid, to: &Oid| {
        read(git, from, to).map_err(AfterPublishError::Git)
    };
    let mapping = replayed_position(
        &source,
        existing.as_ref(),
        &list(Git::rev_list, base, tip)?,
        &list(Git::first_parent_list, base, tip)?,
        &list(Git::rev_list, origin, new_tip)?,
        &list(Git::first_parent_list, origin, new_tip)?,
    );
    Ok(match mapping {
        RecordMapping::NotOwed => RecordRead::NotOwed,
        RecordMapping::Mapped(new) => RecordRead::Move(RefMove { refname, old: existing, new }),
        RecordMapping::Unmappable => RecordRead::Unmappable { source },
    })
}
