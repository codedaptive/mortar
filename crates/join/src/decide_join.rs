//! decide_join: the join decision from the zone tip, the request tip and their fork point.
//! decide_join_tests covers each rule.

use crate::join_decision::JoinDecision;
use crate::join_read::JoinRead;

/// Zone refs only move forward, so the merge base is the request's fork point. The rules, in this order:
/// fork is the request tip -> Joined (so equal tips are Joined); fork is the zone tip -> FastForward from
/// the zone tip to the request tip; any other fork -> Replay onto the zone tip from that fork; no fork ->
/// Unrelated.
pub(crate) fn decide_join(read: &JoinRead) -> JoinDecision {
    match &read.fork {
        Some(fork) if *fork == read.request_tip => JoinDecision::Joined,
        Some(fork) if *fork == read.zone_tip => {
            JoinDecision::FastForward { from: read.zone_tip.clone(), to: read.request_tip.clone() }
        }
        Some(fork) => JoinDecision::Replay { base: fork.clone(), onto: read.zone_tip.clone() },
        None => JoinDecision::Unrelated,
    }
}
