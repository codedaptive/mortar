//! decide_zone_case: which case one zone is in after a publish, from ids already read. decide_zone_case_tests
//! covers it.

use crate::after_publish::{SquashOfRead, ZoneCase};
use gitio::Oid;

/// `base` is the merge base of `origin` and the zone `tip`. A base at origin is Current for every zone, publishing
/// or not, so a repeat moves nothing. A zone that did not publish: Advance when its tip is the base (it holds
/// nothing origin lacks), else Replay from the base. The publishing zone, by the commit it holds for the squash's
/// Mortar-Squash-Of (that commit, or its recorded replay): Advance when that is its tip (no join since the
/// squash), else Replay from it, so only the later joins are replayed; a squash with no Mortar-Squash-Of is
/// NoSquashOf, and one whose named commit the zone holds in neither form is SquashOfNotInZone naming that commit.
pub(crate) fn decide_zone_case(tip: &Oid, base: &Oid, origin: &Oid, squash_of: &SquashOfRead) -> ZoneCase {
    if base == origin {
        return ZoneCase::Current;
    }
    match squash_of {
        SquashOfRead::NotPublishing if tip == base => ZoneCase::Advance,
        SquashOfRead::NotPublishing => ZoneCase::Replay { base: base.clone() },
        SquashOfRead::Missing => ZoneCase::NoSquashOf,
        SquashOfRead::Found { commit, held: None, .. } => ZoneCase::SquashOfNotInZone { squash_of: commit.clone() },
        SquashOfRead::Found { held: Some(held), .. } if held == tip => ZoneCase::Advance,
        SquashOfRead::Found { held: Some(held), .. } => ZoneCase::Replay { base: held.clone() },
    }
}
