//! SquashOfRead: what move_zone read of the published squash's Mortar-Squash-Of. A declaration only (S-7);
//! decide_zone_case_tests exercises each variant.

use gitio::Oid;

/// NotPublishing: the zone did not publish, so nothing was read. Missing: no squash of this zone on origin's line
/// carries Mortar-Squash-Of. Found: the zone tip it names (`commit`) and `held`, the zone's commit standing for it:
/// `commit` itself when it is the zone tip or an ancestor of it; its replayed counterpart when an earlier
/// after_publish replayed it onto another zone's origin; None when the zone holds neither.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SquashOfRead {
    NotPublishing,
    Missing,
    Found { commit: Oid, held: Option<Oid> },
}
