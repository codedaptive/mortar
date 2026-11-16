//! The publishing zone landed as a merge. Zone `land` is squashed on the base; zone `b` is then squashed,
//! landed and published first, so `land`'s squash lands as a merge commit [origin, squash] and origin is that merge,
//! whose message carries no trailers. after_publish must find the squash's Mortar-Squash-Of through the merge's
//! second parent and put the zone on origin.

use crate::fixture::{land_and_publish, made, refs, zone, Fixture, ACTOR, B_FILES, PUBLISHED, PUBLISHED_FILES};
use land::{after_publish, ZoneMove};

#[test]
fn the_publishing_zone_landed_as_a_merge_is_found_through_the_second_parent() {
    let fx = Fixture::new();
    let tip = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let land_squash = made(&fx, PUBLISHED);
    let b_origin = land_and_publish(&fx, &made(&fx, "b"));
    let origin = land_and_publish(&fx, &land_squash);
    // The landing went through a merge: origin is [b's published squash, land's squash].
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![b_origin, land_squash.commit.clone()]);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR).unwrap();
    assert_eq!(
        moves,
        vec![ZoneMove::Moved { zone: PUBLISHED.to_string(), old: tip, new: origin.clone(), rebase: vec![] }]
    );
    assert_eq!(fx.branch("zone/land"), origin);
    assert_eq!(fx.branch("main"), origin);
    assert_eq!(fx.remote_main(), Some(origin));
}
