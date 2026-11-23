//! decide_base_join: the join decision for a request made on a partner merge. decide_base_join_tests covers
//! each rule.

use crate::base_join_decision::BaseJoinDecision;
use crate::base_join_read::BaseJoinRead;

/// The rules, in this order: the request tip in the zone -> Joined; the base's parents are exactly the zone
/// tip followed by the partners' current tips in the record's order -> ReplayOntoZone from the base onto
/// the zone tip; anything else (the zone or a partner moved, the order differs) -> Remerge.
pub(crate) fn decide_base_join(read: &BaseJoinRead) -> BaseJoinDecision {
    if read.request_in_zone {
        return BaseJoinDecision::Joined;
    }
    let unchanged = read.base_parents.split_first().is_some_and(|(first, rest)| {
        *first == read.zone_tip && rest == read.partner_tips.as_slice()
    });
    if unchanged {
        return BaseJoinDecision::ReplayOntoZone { base: read.base.clone(), onto: read.zone_tip.clone() };
    }
    BaseJoinDecision::Remerge
}
