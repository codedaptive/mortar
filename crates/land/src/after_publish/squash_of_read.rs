//! SquashOfRead: what move_zone read of the published squash's Mortar-Squash-Of. A declaration only (S-7);
//! decide_zone_case_tests exercises each variant.

use gitio::Oid;

/// NotPublishing: the zone did not publish, so nothing was read. Missing: no squash of this zone on origin's line
/// carries Mortar-Squash-Of. Found: the zone tip it names (`commit`) and `held`, the zone's commit standing for it:
/// the value of its replay record `refs/mortar/replayed/<zone>/<commit>` when the zone tip holds that; else `commit`
/// itself when it is the zone tip or an ancestor of it; None when the zone holds neither. Superseded: the zone holds
/// the named commit (`commit`) in neither form, but it exists and is at or behind the existing source an
/// earlier-published squash of this zone named, found on the first-parent line from the merge base back to what the
/// publishing squash's first parent reaches, so the
/// publish brings the zone nothing it lacks (rule O, S-121).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SquashOfRead {
    NotPublishing,
    Missing,
    Found { commit: Oid, held: Option<Oid> },
    Superseded { commit: Oid },
}
