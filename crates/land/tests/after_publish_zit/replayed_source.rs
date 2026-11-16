//! The publishing zone whose squash source an earlier after_publish replayed. Zone `land` is squashed at A1 and
//! the squash kept; zone `b` is squashed, landed and published first, and after_publish(b, others=[land]) replays A1
//! onto b's origin as A1' (fixture replayed). The kept squash, still naming A1 in its Mortar-Squash-Of, then lands
//! as a merge and publishes. after_publish(land) must take A1', the replay of A1 that after_publish(b) recorded, as
//! the zone's counterpart of A1: put the zone on origin when A1' is its tip, and replay only a request joined after
//! A1' onto origin when there is one. Origin is never rewritten.

use crate::fixture::{
    land_and_publish, refs, replayed, request, zone, Fixture, Replayed, ACTOR, JOINED_FILES, PUBLISHED,
};
use land::{after_publish, ZoneMove};

/// Fixture replayed, checked to replay A1 to a new commit carrying A1's message.
fn replayed_a1(fx: &Fixture) -> Replayed {
    let replayed = replayed(fx);
    assert_ne!(replayed.a1_replayed, replayed.a1);
    let message = |commit| fx.git.commit_info(commit).unwrap().message;
    assert_eq!(message(&replayed.a1_replayed), message(&replayed.a1));
    replayed
}

#[test]
fn the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin() {
    let fx = Fixture::new();
    let Replayed { land_squash, b_origin, a1_replayed, .. } = replayed_a1(&fx);
    let origin = land_and_publish(&fx, &land_squash);
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![b_origin.clone(), land_squash.commit.clone()]);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[zone("b", vec![])], ACTOR);
    assert!(moves.is_ok(), "{moves:?}");
    assert_eq!(
        moves.unwrap(),
        vec![
            ZoneMove::Moved { zone: PUBLISHED.to_string(), old: a1_replayed, new: origin.clone(), rebase: vec![] },
            ZoneMove::Moved { zone: "b".to_string(), old: b_origin, new: origin.clone(), rebase: vec![] },
        ]
    );
    assert_eq!(fx.branch("zone/land"), origin);
    assert_eq!(fx.branch("zone/b"), origin);
    assert_eq!(fx.branch("main"), origin);
    assert_eq!(fx.remote_main(), Some(origin));
}

#[test]
fn a_request_joined_after_the_replay_is_replayed_onto_origin() {
    let fx = Fixture::new();
    let Replayed { land_squash, a1_replayed, .. } = replayed_a1(&fx);
    let joined = fx.join_onto(PUBLISHED, &a1_replayed, &JOINED_FILES, "joined after the replay");
    let open = request("WO-land-17", &a1_replayed, &joined);
    let origin = land_and_publish(&fx, &land_squash);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![open.clone()]), &[], ACTOR);
    assert!(moves.is_ok(), "{moves:?}");
    let moves = moves.unwrap();
    let [ZoneMove::Moved { zone: name, old, new, rebase }] = moves.as_slice() else {
        panic!("the published zone must move: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &joined, &vec![open]));
    assert_eq!(fx.branch("zone/land"), *new);
    // Only the join after A1' is replayed: A1' is the counterpart of the squash's source, whose content origin
    // holds through the squash.
    assert_eq!(fx.git.commit_info(new).unwrap().parents, vec![origin.clone()]);
    assert_eq!(fx.files(new), ["a.txt", "b.txt", "j.txt", "p.txt"]);
    assert_eq!(fx.file(new, "j.txt"), "joined\n");
    assert_eq!(fx.file(new, "p.txt"), "published\n");
    assert_eq!(fx.branch("main"), origin);
    assert_eq!(fx.remote_main(), Some(origin));
}
