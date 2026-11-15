//! SquashOfRead: what move_zone read of the published squash's Mortar-Squash-Of. A declaration only (S-7);
//! decide_zone_case_tests exercises each variant.

use gitio::Oid;

/// NotPublishing: the zone did not publish, so nothing was read. Missing: no squash of this zone on origin's line
/// carries Mortar-Squash-Of. Found: the zone tip it names and whether that commit is in the zone (an ancestor of
/// the zone tip, or the tip itself).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SquashOfRead {
    NotPublishing,
    Missing,
    Found { commit: Oid, in_zone: bool },
}
