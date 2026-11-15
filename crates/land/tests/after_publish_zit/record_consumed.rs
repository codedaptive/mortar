//! The replay record consumed by the publish it serves (S-118 (1)). Zone `land` is squashed at A1 and the squash
//! kept; zone `b` is squashed, landed and published first, and after_publish(b, others=[land]) replays A1 onto b's
//! origin as A1' and records `refs/mortar/replayed/land/<A1>` = A1'. The kept squash, naming A1 in its
//! Mortar-Squash-Of, then lands as a merge and publishes. after_publish(land) moves zone `land` past A1 and deletes
//! the record in the same op: when A1' is the zone tip (put on origin) and when a join J sits on A1' (J replayed
//! onto origin). Undo of that op, scoped to the zone branch, brings the zone and the record back together; a repeat
//! after the consuming publish moves nothing and writes no op. Origin is never rewritten.

use crate::fixture::{
    land_and_publish, made, origin_untouched, refs, zone, Fixture, ACTOR, B_FILES, PUBLISHED, PUBLISHED_FILES,
};
use gitio::Oid;
use land::{after_publish, Squash, ZoneMove};
use oplog::{undo, ApplyOutcome, OpCaller};

/// A join on the replayed A1': the files of A1' (b.txt carried in by the replay) plus j.txt.
const JOINED_FILES: [(&str, &str); 4] =
    [("a.txt", "one\nTWO\nthree\n"), ("b.txt", "b\n"), ("j.txt", "joined\n"), ("p.txt", "published\n")];

/// Up to the replay: land's squash kept at A1, b published, after_publish(b, [land]) checked to replay A1 to A1'
/// and to record A1' at the record ref. Answers (land's kept squash, the record ref, A1').
fn replayed(fx: &Fixture) -> (Squash, String, Oid) {
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let land_squash = made(fx, PUBLISHED);
    let b_origin = land_and_publish(fx, &made(fx, "b"));
    let moves = after_publish(&fx.git, &refs(), &zone("b", vec![]), &[zone(PUBLISHED, vec![])], ACTOR).unwrap();
    let ZoneMove::Moved { zone: name, old, new: a1_replayed, rebase } = &moves[1] else {
        panic!("zone land must be replayed onto b's origin: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &a1, &vec![]));
    assert_eq!(fx.git.commit_info(a1_replayed).unwrap().parents, vec![b_origin]);
    let record = format!("refs/mortar/replayed/{PUBLISHED}/{}", a1.as_str());
    assert_eq!(fx.git.read_ref(&record).unwrap(), Some(a1_replayed.clone()));
    (land_squash, record, a1_replayed.clone())
}

/// Up to the consuming publish: land's kept squash landed and published, after_publish(land, []) checked to put
/// the zone on origin. Answers (the record ref, A1', origin).
fn consumed(fx: &Fixture) -> (String, Oid, Oid) {
    let (land_squash, record, a1_replayed) = replayed(fx);
    let origin = land_and_publish(fx, &land_squash);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected = vec![ZoneMove::Moved {
        zone: PUBLISHED.to_string(),
        old: a1_replayed.clone(),
        new: origin.clone(),
        rebase: vec![],
    }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    (record, a1_replayed, origin)
}

#[test]
fn the_publish_that_consumes_the_record_deletes_it_and_undo_brings_it_back() {
    let fx = Fixture::new();
    let (record, a1_replayed, origin) = consumed(&fx);
    assert_eq!(fx.git.read_ref(&record).unwrap(), None);
    assert_eq!(fx.branch("zone/land"), origin);
    origin_untouched(&fx, &origin);
    // Scoped to the zone branch: the last done op moving it is after_publish(land), which also deleted the record.
    let caller = OpCaller {
        key: "zit-undo-consuming-publish".to_string(),
        actor: ACTOR.to_string(),
        reason: "undo the publish that consumed the replay record".to_string(),
    };
    let undone = undo(&fx.git, &fx.common, &[format!("refs/heads/zone/{PUBLISHED}")], &caller, &[]);
    assert!(matches!(&undone, Ok(ApplyOutcome::Done(_))), "{undone:?}");
    assert_eq!(fx.branch("zone/land"), a1_replayed);
    assert_eq!(fx.git.read_ref(&record).unwrap(), Some(a1_replayed));
    origin_untouched(&fx, &origin);
}

#[test]
fn a_replayed_publish_deletes_the_record_too() {
    let fx = Fixture::new();
    let (land_squash, record, a1_replayed) = replayed(&fx);
    let joined = fx.join_onto(PUBLISHED, &a1_replayed, &JOINED_FILES, "joined after the replay");
    let origin = land_and_publish(&fx, &land_squash);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    assert!(moves.is_ok(), "{moves:?}");
    let moves = moves.unwrap();
    let [ZoneMove::Moved { zone: name, old, new, rebase }] = moves.as_slice() else {
        panic!("the published zone must move: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &joined, &vec![]));
    assert_eq!(fx.git.read_ref(&record).unwrap(), None);
    assert_eq!(fx.branch("zone/land"), *new);
    assert_eq!(fx.git.commit_info(new).unwrap().parents, vec![origin.clone()]);
    origin_untouched(&fx, &origin);
}

#[test]
fn a_repeat_after_the_consuming_publish_moves_nothing() {
    let fx = Fixture::new();
    let (record, _, origin) = consumed(&fx);
    let records = fx.after_publish_records();
    let repeat = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected = vec![ZoneMove::Current { zone: PUBLISHED.to_string(), tip: origin.clone() }];
    assert!(matches!(&repeat, Ok(answer) if *answer == expected), "{repeat:?}");
    assert_eq!(fx.git.read_ref(&record).unwrap(), None);
    assert_eq!(fx.after_publish_records(), records);
    assert_eq!(fx.branch("zone/land"), origin);
    origin_untouched(&fx, &origin);
}
