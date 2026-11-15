//! The publishing zone whose squash source an earlier after_publish replayed. Zone `land` is squashed at A1 and
//! the squash kept; zone `b` is squashed, landed and published first, and after_publish(b, others=[land]) replays A1
//! onto b's origin as A1'. The kept squash, still naming A1 in its Mortar-Squash-Of, then lands as a merge and
//! publishes. after_publish(land) must take A1', the replay of A1 that after_publish(b) recorded, as the zone's
//! counterpart of A1: put the zone on origin when A1' is its tip, and replay only a request joined after A1' onto
//! origin when there is one. Origin is never rewritten.

use crate::fixture::{
    land_and_publish, made, refs, request, zone, Fixture, ACTOR, B_FILES, PUBLISHED, PUBLISHED_FILES,
};
use gitio::Oid;
use land::{after_publish, Squash, ZoneMove};

/// A request joined on the replayed A1': the files of A1' (b.txt carried in by the replay) plus j.txt.
const JOINED_FILES: [(&str, &str); 4] =
    [("a.txt", "one\nTWO\nthree\n"), ("b.txt", "b\n"), ("j.txt", "joined\n"), ("p.txt", "published\n")];

/// Up to the replay: land's squash kept, b published, after_publish(b, [land]) checked to replay A1 to A1'.
/// Answers (land's kept squash, b's origin, A1').
fn replayed(fx: &Fixture) -> (Squash, Oid, Oid) {
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let land_squash = made(fx, PUBLISHED);
    let b_origin = land_and_publish(fx, &made(fx, "b"));
    let moves = after_publish(&fx.git, &refs(), &zone("b", vec![]), &[zone(PUBLISHED, vec![])], ACTOR).unwrap();
    let ZoneMove::Moved { zone: name, old, new: a1_replayed, rebase } = &moves[1] else {
        panic!("zone land must be replayed onto b's origin: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &a1, &vec![]));
    assert_ne!(*a1_replayed, a1);
    assert_eq!(fx.git.commit_info(a1_replayed).unwrap().parents, vec![b_origin.clone()]);
    assert_eq!(fx.git.commit_info(a1_replayed).unwrap().message, fx.git.commit_info(&a1).unwrap().message);
    (land_squash, b_origin, a1_replayed.clone())
}

#[test]
fn the_publishing_zone_whose_source_an_earlier_publish_replayed_is_put_on_origin() {
    let fx = Fixture::new();
    let (land_squash, b_origin, a1_replayed) = replayed(&fx);
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
    let (land_squash, _, a1_replayed) = replayed(&fx);
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
