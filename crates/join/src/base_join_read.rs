//! BaseJoinRead: what join_on_base reads before deciding. A declaration only; decide_base_join_tests
//! exercises it.

use gitio::Oid;

/// The zone ref's tip, whether the request tip is already in the zone, the request's recorded base, that
/// base's parents, and each partner's current tip in the record's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BaseJoinRead {
    pub(crate) zone_tip: Oid,
    pub(crate) request_in_zone: bool,
    pub(crate) base: Oid,
    pub(crate) base_parents: Vec<Oid>,
    pub(crate) partner_tips: Vec<Oid>,
}
