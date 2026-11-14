//! decide_join: each of the four rules, and equal tips as Joined.

use crate::decide_join::decide_join;
use crate::join_decision::JoinDecision;
use crate::join_read::JoinRead;
use gitio::Oid;

const ZONE: &str = "1111111111111111111111111111111111111111";
const REQUEST: &str = "2222222222222222222222222222222222222222";
const FORK: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn read(zone_tip: &str, request_tip: &str, fork: Option<&str>) -> JoinRead {
    JoinRead { zone_tip: oid(zone_tip), request_tip: oid(request_tip), fork: fork.map(oid) }
}

#[test]
fn equal_tips_are_joined() {
    assert_eq!(decide_join(&read(ZONE, ZONE, Some(ZONE))), JoinDecision::Joined);
}

#[test]
fn fork_at_the_request_tip_with_the_zone_ahead_is_joined() {
    assert_eq!(decide_join(&read(ZONE, REQUEST, Some(REQUEST))), JoinDecision::Joined);
}

#[test]
fn fork_at_the_zone_tip_fast_forwards_from_zone_tip_to_request_tip() {
    assert_eq!(
        decide_join(&read(ZONE, REQUEST, Some(ZONE))),
        JoinDecision::FastForward {
            from: oid("1111111111111111111111111111111111111111"),
            to: oid("2222222222222222222222222222222222222222"),
        }
    );
}

#[test]
fn fork_elsewhere_replays_from_the_fork_onto_the_zone_tip() {
    assert_eq!(
        decide_join(&read(ZONE, REQUEST, Some(FORK))),
        JoinDecision::Replay {
            base: oid("3333333333333333333333333333333333333333"),
            onto: oid("1111111111111111111111111111111111111111"),
        }
    );
}

#[test]
fn no_fork_is_unrelated() {
    assert_eq!(decide_join(&read(ZONE, REQUEST, None)), JoinDecision::Unrelated);
}
