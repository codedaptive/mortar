//! JoinDecision: what join does with what it read. A declaration only; decide_join_tests exercises it.

use gitio::Oid;

/// The one move (or none) a join makes, chosen by decide_join.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum JoinDecision {
    /// The request tip is already in the zone; nothing moves.
    Joined,
    /// The zone ref moves forward from the zone tip to the request tip.
    FastForward { from: Oid, to: Oid },
    /// The request's commits after `base` are replayed onto the zone tip `onto`.
    Replay { base: Oid, onto: Oid },
    /// The request and the zone share no history; join refuses.
    Unrelated,
}
