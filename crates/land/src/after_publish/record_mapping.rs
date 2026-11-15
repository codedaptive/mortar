//! RecordMapping: what a replay says of the zone commit standing for a squash source. A declaration only (S-7);
//! replayed_position_tests exercises each variant.

use gitio::Oid;

/// NotOwed: the commit looked up (the source's record, else the source) is not in the replayed range, so the replay
/// owes no record. Mapped: the replay's commit standing for it. Unmappable: it is in the range, but the replay dropped
/// a commit or a range is not one first-parent chain, so the commit standing for it cannot be named (S-120).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RecordMapping {
    NotOwed,
    Mapped(Oid),
    Unmappable,
}
