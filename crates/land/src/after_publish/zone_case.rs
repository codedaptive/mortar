//! ZoneCase: what to do with one zone after a publish. A declaration only (S-7); decide_zone_case_tests exercises
//! each variant.

use gitio::Oid;

/// Current: the zone already holds origin; nothing to move. Advance: the zone has nothing of its own past origin's
/// reach, so one op puts it on origin. Replay: the zone's commits after `base` are replayed onto origin.
/// NoSquashOf and SquashOfNotInZone: the publishing zone's squash names no zone tip, or one the zone holds neither
/// itself nor as its recorded replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ZoneCase {
    Current,
    Advance,
    Replay { base: Oid },
    NoSquashOf,
    SquashOfNotInZone { squash_of: Oid },
}
