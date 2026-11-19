//! read_publishing: what the publishing zone holds for the squash just published (a held record first, then the
//! older-squash check).
//! No unit test: it only reads git; choose_held_tests and decide_zone_case_tests cover what is decided from
//! its answer. Covered by tests/after_publish_zit: (older_squash.rs)
//! an_older_squash_published_after_the_newer_one_leaves_the_zone_current_and_writes_no_op (Superseded) and
//! a_squash_whose_source_is_not_behind_the_published_one_is_refused (the source not behind the earlier squash's:
//! Found with nothing held); (records_deleted.rs)
//! a_current_publish_deletes_only_the_stale_record_of_its_source_in_a_records_only_op (Superseded with a record not
//! held); (replay_provenance.rs) a_re_save_with_the_same_metadata_after_the_replay_is_refused_not_dropped (no
//! earlier squash: Found with nothing held); (zone_moves.rs) the_publishing_zone_with_no_later_joins_is_put_on_origin
//! and (merge_landing.rs) the_publishing_zone_landed_as_a_merge_is_found_through_the_second_parent (Found, held).

use crate::after_publish::{read_held, read_squash_of, AfterPublishError, SquashOfRead};
use gitio::{Git, Oid};

/// `base` is the merge base of `origin` and the zone `tip`. The zone's squash S on origin's line since `base`
/// (read_squash_of; none: Missing) names the source A; read_held answers what the zone holds for A. When it holds
/// A in neither form, the older-squash check: the newest earlier squash of this zone on the first-parent line from
/// `base` back to what S's first parent reaches names P; when A and P both exist and A is P or an ancestor of it, A
/// is at or behind a source already published, so S carries nothing new, and the answer is Superseded with A.
/// Otherwise read_held's answer stands. A is read only when no held record answers (read_held), and the older-squash
/// check runs only then. A failed git run is Git.
pub(crate) fn read_publishing(
    git: &Git,
    zone: &str,
    origin: &Oid,
    tip: &Oid,
    base: &Oid,
) -> Result<SquashOfRead, AfterPublishError> {
    let Some((squash, source)) = read_squash_of(git, origin, tip, zone)? else {
        return Ok(SquashOfRead::Missing);
    };
    let read = read_held(git, zone, &source, tip)?;
    if !matches!(read, SquashOfRead::Found { held: None, .. }) || !behind_earlier(git, zone, &squash, &source, base)? {
        return Ok(read);
    }
    Ok(SquashOfRead::Superseded { commit: source })
}

/// The older-squash check: the earlier squash's source P read on the line from `base` back to what S's first parent
/// reaches (read_squash_of with `base` as its origin and that parent as its zone tip), then whether `source`
/// and P both exist and `source` is P or an ancestor of it. No parent, no earlier squash, or a missing object: false.
fn behind_earlier(git: &Git, zone: &str, squash: &Oid, source: &Oid, base: &Oid) -> Result<bool, AfterPublishError> {
    let Some(parent) = git.commit_info(squash).map_err(AfterPublishError::Git)?.parents.into_iter().next() else {
        return Ok(false);
    };
    let Some((_, earlier)) = read_squash_of(git, base, &parent, zone)? else {
        return Ok(false);
    };
    for commit in [source, &earlier] {
        if git.rev_parse(commit.as_str()).map_err(AfterPublishError::Git)?.is_none() {
            return Ok(false);
        }
    }
    git.is_ancestor(source, &earlier).map_err(AfterPublishError::Git)
}
