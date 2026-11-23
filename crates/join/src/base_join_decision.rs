//! BaseJoinDecision: what join_on_base does with what it read. A declaration only; decide_base_join_tests
//! exercises it.

use gitio::Oid;

/// The one path a join on a partner base takes, chosen by decide_base_join.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BaseJoinDecision {
    /// The request tip is already in the zone; nothing moves.
    Joined,
    /// Zone and partners are where the base had them: the request's commits after `base` are replayed onto
    /// the zone tip `onto`, and the zone and request refs both move to the result.
    ReplayOntoZone { base: Oid, onto: Oid },
    /// The zone or a partner moved: the partner merge is made again and the request rebased onto it.
    Remerge,
}
