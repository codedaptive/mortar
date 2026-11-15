//! decide_zone_case: each rule with literal ids, in the contract's order: Current, then the zone that did not
//! publish (Advance, Replay from the base), then the publishing zone (NoSquashOf, SquashOfNotInZone, Advance,
//! Replay from the squash-of).

use super::{decide_zone_case, SquashOfRead, ZoneCase};
use gitio::Oid;

const TIP: &str = "1111111111111111111111111111111111111111";
const BASE: &str = "2222222222222222222222222222222222222222";
const ORIGIN: &str = "3333333333333333333333333333333333333333";
const SQUASH_OF: &str = "4444444444444444444444444444444444444444";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn found(commit: &str, in_zone: bool) -> SquashOfRead {
    SquashOfRead::Found { commit: oid(commit), in_zone }
}

#[test]
fn a_base_at_origin_is_current_for_every_zone_publishing_or_not() {
    let reads = [
        SquashOfRead::NotPublishing,
        SquashOfRead::Missing,
        found(SQUASH_OF, true),
        found(SQUASH_OF, false),
        found(TIP, true),
    ];
    for read in &reads {
        assert_eq!(decide_zone_case(&oid(TIP), &oid(ORIGIN), &oid(ORIGIN), read), ZoneCase::Current, "{read:?}");
    }
}

#[test]
fn a_zone_at_origin_itself_is_current() {
    let read = SquashOfRead::NotPublishing;
    assert_eq!(decide_zone_case(&oid(ORIGIN), &oid(ORIGIN), &oid(ORIGIN), &read), ZoneCase::Current);
}

#[test]
fn a_zone_not_publishing_whose_tip_is_the_base_advances() {
    let read = SquashOfRead::NotPublishing;
    assert_eq!(decide_zone_case(&oid(BASE), &oid(BASE), &oid(ORIGIN), &read), ZoneCase::Advance);
}

#[test]
fn a_zone_not_publishing_with_commits_past_the_base_replays_from_the_base() {
    let read = SquashOfRead::NotPublishing;
    assert_eq!(decide_zone_case(&oid(TIP), &oid(BASE), &oid(ORIGIN), &read), ZoneCase::Replay { base: oid(BASE) });
}

#[test]
fn the_publishing_zone_with_no_squash_of_is_no_squash_of() {
    let read = SquashOfRead::Missing;
    assert_eq!(decide_zone_case(&oid(TIP), &oid(BASE), &oid(ORIGIN), &read), ZoneCase::NoSquashOf);
}

#[test]
fn a_squash_of_outside_the_zone_is_refused_naming_it() {
    assert_eq!(
        decide_zone_case(&oid(TIP), &oid(BASE), &oid(ORIGIN), &found(SQUASH_OF, false)),
        ZoneCase::SquashOfNotInZone { squash_of: oid(SQUASH_OF) }
    );
}

#[test]
fn a_squash_of_naming_the_tip_outside_the_zone_is_still_refused() {
    assert_eq!(
        decide_zone_case(&oid(TIP), &oid(BASE), &oid(ORIGIN), &found(TIP, false)),
        ZoneCase::SquashOfNotInZone { squash_of: oid(TIP) }
    );
}

#[test]
fn the_publishing_zone_whose_tip_is_the_squash_of_advances() {
    assert_eq!(decide_zone_case(&oid(TIP), &oid(BASE), &oid(ORIGIN), &found(TIP, true)), ZoneCase::Advance);
}

#[test]
fn the_publishing_zone_moved_since_the_squash_replays_from_the_squash_of_not_the_base() {
    assert_eq!(
        decide_zone_case(&oid(TIP), &oid(BASE), &oid(ORIGIN), &found(SQUASH_OF, true)),
        ZoneCase::Replay { base: oid(SQUASH_OF) }
    );
}
