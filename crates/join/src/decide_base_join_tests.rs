//! decide_base_join: each rule as a row, the in-zone rule ahead of the others, and the near misses of the
//! unchanged rule (zone moved, a partner moved, partners swapped, a single-parent base) as Remerge.

use crate::base_join_decision::BaseJoinDecision;
use crate::base_join_read::BaseJoinRead;
use crate::decide_base_join::decide_base_join;
use gitio::Oid;

const ZONE: &str = "1111111111111111111111111111111111111111";
const OLD_ZONE: &str = "2222222222222222222222222222222222222222";
const BASE: &str = "3333333333333333333333333333333333333333";
const P1: &str = "4444444444444444444444444444444444444444";
const P2: &str = "5555555555555555555555555555555555555555";
const NEW_P2: &str = "6666666666666666666666666666666666666666";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn read(request_in_zone: bool, base_parents: &[&str], partner_tips: &[&str]) -> BaseJoinRead {
    BaseJoinRead {
        zone_tip: oid(ZONE),
        request_in_zone,
        base: oid(BASE),
        base_parents: base_parents.iter().map(|hex| oid(hex)).collect(),
        partner_tips: partner_tips.iter().map(|hex| oid(hex)).collect(),
    }
}

fn replay_onto_zone() -> BaseJoinDecision {
    BaseJoinDecision::ReplayOntoZone {
        base: oid("3333333333333333333333333333333333333333"),
        onto: oid("1111111111111111111111111111111111111111"),
    }
}

#[test]
fn decides_each_rule_in_order() {
    let rows = [
        ("in zone, base unchanged", read(true, &[ZONE, P1], &[P1]), BaseJoinDecision::Joined),
        ("in zone, zone moved", read(true, &[OLD_ZONE, P1], &[P1]), BaseJoinDecision::Joined),
        ("unchanged, one partner", read(false, &[ZONE, P1], &[P1]), replay_onto_zone()),
        ("unchanged, two partners", read(false, &[ZONE, P1, P2], &[P1, P2]), replay_onto_zone()),
        ("zone moved", read(false, &[OLD_ZONE, P1], &[P1]), BaseJoinDecision::Remerge),
        ("one partner moved", read(false, &[ZONE, P1, P2], &[P1, NEW_P2]), BaseJoinDecision::Remerge),
        ("partner order swapped", read(false, &[ZONE, P2, P1], &[P1, P2]), BaseJoinDecision::Remerge),
        ("base with a single parent", read(false, &[ZONE], &[P1]), BaseJoinDecision::Remerge),
    ];
    for (name, read, expected) in rows {
        assert_eq!(decide_base_join(&read), expected, "{name}");
    }
}
