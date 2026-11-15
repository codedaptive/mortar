//! SquashOfRead: what move_zone read of the published squash's Mortar-Squash-Of. A declaration only (S-7);
//! decide_zone_case_tests and consumed_record_tests exercise each variant.

use gitio::Oid;

/// NotPublishing: the zone did not publish, so nothing was read. Missing: no squash of this zone on origin's line
/// carries Mortar-Squash-Of. Found: the zone tip it names (`commit`) and `held`, the zone's commit standing for it:
/// the replay of `commit` an earlier after_publish recorded when it replayed the zone onto another zone's origin,
/// when the zone tip holds it; else `commit` itself when it is the zone tip or an ancestor of it; None when the zone
/// holds neither. `record` is the value of that replay record, `refs/mortar/replayed/<zone>/<commit>`, whether or not
/// the zone tip holds it; None when there is no record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SquashOfRead {
    NotPublishing,
    Missing,
    Found { commit: Oid, held: Option<Oid>, record: Option<Oid> },
}
