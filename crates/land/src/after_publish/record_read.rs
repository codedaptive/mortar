//! RecordRead: what read_record_move read for a zone's replay record. A declaration only (S-7);
//! clean_replay_move_tests exercises each variant.

use gitio::Oid;
use oplog::RefMove;

/// NotOwed: no squash out, no Mortar-Squash-Of for the zone, or the replay did not carry the commit standing for
/// its source; no record moves. Move: the record set to the replay's commit standing for the source. Unmappable:
/// the replay carried that commit but dropped one or is not one chain, so the commit standing for `source` (the
/// squash's Mortar-Squash-Of) cannot be named (S-120).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RecordRead {
    NotOwed,
    Move(RefMove),
    Unmappable { source: Oid },
}
