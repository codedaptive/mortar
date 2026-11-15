//! The cases of SPEC-land 05-after-publish.md §Tests, and a repeat: after land's own squash, land and publish of
//! zone `land`, another zone is replayed onto the new origin with its open requests listed to rebase; the publishing
//! zone with no join since its squash is put on origin, and one with a later join has only that join replayed onto
//! origin; a zone whose commit conflicts with the published squash is left in place and listed with its file while
//! another zone in the same call still moves; a repeat answers Current for every zone and writes no op. Every case
//! checks that origin, local and remote, is where publish left it.

use crate::fixture::{refs, request, zone, Fixture, ACTOR, B_FILES, C_FILES, PUBLISHED, PUBLISHED_FILES};
use gitio::Oid;
use land::{after_publish, ZoneMove};
use oplog::{OpPhase, RefMove};
use std::path::PathBuf;

/// The published zone's later join: PUBLISHED_FILES plus later.txt.
const LATER_FILES: [(&str, &str); 3] =
    [("a.txt", "one\nTWO\nthree\n"), ("later.txt", "later\n"), ("p.txt", "published\n")];

/// Origin, local and remote, is still the published commit.
fn origin_unchanged(fx: &Fixture, origin: &Oid) {
    assert_eq!(fx.branch("main"), *origin);
    assert_eq!(fx.remote_main(), Some(origin.clone()));
}

/// The moves of the log's done after-publish records, in log order.
fn done_moves(fx: &Fixture) -> Vec<Vec<RefMove>> {
    let records = fx.after_publish_records().into_iter().filter(|record| record.phase == OpPhase::Done);
    records.map(|record| record.moves).collect()
}

fn zone_move(refname: &str, old: &Oid, new: &Oid) -> Vec<RefMove> {
    vec![RefMove { refname: refname.to_string(), old: Some(old.clone()), new: new.clone() }]
}

#[test]
fn another_zone_is_replayed_onto_the_new_origin() {
    let fx = Fixture::new();
    let published_tip = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let b_tip = fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let b_request = request("WO-b-1", &b_tip, &b_tip);
    let origin = fx.publish();
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[zone("b", vec![b_request.clone()])], ACTOR)
        .unwrap();
    let ZoneMove::Moved { zone: name, old, new, rebase } = &moves[1] else {
        panic!("zone b must move: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), ("b", &b_tip, &vec![b_request]));
    assert_eq!(fx.branch("zone/b"), *new);
    assert_eq!(fx.git.commit_info(new).unwrap().parents, vec![origin.clone()]);
    assert_eq!(fx.files(new), ["a.txt", "b.txt", "p.txt"]);
    assert_eq!(fx.file(new, "b.txt"), "b\n");
    assert_eq!(fx.file(new, "a.txt"), "one\nTWO\nthree\n");
    assert_eq!(
        done_moves(&fx),
        vec![zone_move("refs/heads/zone/land", &published_tip, &origin), zone_move("refs/heads/zone/b", &b_tip, new)]
    );
    origin_unchanged(&fx, &origin);
}

#[test]
fn the_publishing_zone_with_no_later_joins_is_put_on_origin() {
    let fx = Fixture::new();
    let tip = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let open = request("WO-land-9", &tip, &tip);
    let origin = fx.publish();
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![open.clone()]), &[], ACTOR).unwrap();
    assert_eq!(
        moves,
        vec![ZoneMove::Moved { zone: PUBLISHED.to_string(), old: tip.clone(), new: origin.clone(), rebase: vec![open] }]
    );
    assert_eq!(fx.branch("zone/land"), origin);
    assert_eq!(done_moves(&fx), vec![zone_move("refs/heads/zone/land", &tip, &origin)]);
    origin_unchanged(&fx, &origin);
}

#[test]
fn the_publishing_zone_with_a_later_join_is_replayed_onto_origin() {
    let fx = Fixture::new();
    let squashed = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let origin = fx.publish();
    let later = fx.join_onto(PUBLISHED, &squashed, &LATER_FILES, "later join");
    let open = request("WO-land-9", &later, &later);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![open.clone()]), &[], ACTOR).unwrap();
    let [ZoneMove::Moved { zone: name, old, new, rebase }] = moves.as_slice() else {
        panic!("the published zone must move: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &later, &vec![open]));
    assert_eq!(fx.branch("zone/land"), *new);
    // Only the later join is replayed: the squashed commit is origin's own, so it is not replayed again on top.
    assert_eq!(fx.git.commit_info(new).unwrap().parents, vec![origin.clone()]);
    assert_eq!(fx.files(new), ["a.txt", "later.txt", "p.txt"]);
    assert_eq!(fx.file(new, "later.txt"), "later\n");
    assert_eq!(fx.file(new, "a.txt"), "one\nTWO\nthree\n");
    origin_unchanged(&fx, &origin);
}

#[test]
fn a_conflicting_zone_is_left_in_place_and_listed_with_its_files() {
    let fx = Fixture::new();
    fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let c_tip = fx.join_onto("c", &fx.base, &C_FILES, "c work");
    let b_tip = fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let origin = fx.publish();
    let others = [zone("c", vec![]), zone("b", vec![])];
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &others, ACTOR).unwrap();
    assert_eq!(moves.len(), 3, "{moves:?}");
    assert_eq!(
        moves[1],
        ZoneMove::Conflicted {
            zone: "c".to_string(),
            tip: c_tip.clone(),
            commit: c_tip.clone(),
            files: vec![PathBuf::from("a.txt")],
        }
    );
    assert_eq!(fx.branch("zone/c"), c_tip);
    assert_eq!(fx.file(&c_tip, "a.txt"), "one\ntwo-c\nthree\n");
    let ZoneMove::Moved { zone: name, old, new, .. } = &moves[2] else {
        panic!("zone b must still move: {moves:?}");
    };
    assert_eq!((name.as_str(), old), ("b", &b_tip));
    assert_eq!(fx.git.commit_info(new).unwrap().parents, vec![origin.clone()]);
    assert_eq!(fx.file(new, "b.txt"), "b\n");
    origin_unchanged(&fx, &origin);
}

#[test]
fn a_repeat_moves_nothing() {
    let fx = Fixture::new();
    fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let origin = fx.publish();
    let others = [zone("b", vec![])];
    after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &others, ACTOR).unwrap();
    let moved_b = fx.branch("zone/b");
    let records = fx.after_publish_records();
    let repeat = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &others, ACTOR).unwrap();
    assert_eq!(
        repeat,
        vec![
            ZoneMove::Current { zone: PUBLISHED.to_string(), tip: origin.clone() },
            ZoneMove::Current { zone: "b".to_string(), tip: moved_b.clone() },
        ]
    );
    assert_eq!(fx.after_publish_records(), records);
    assert_eq!(records.len(), 4);
    assert_eq!(fx.branch("zone/land"), origin);
    assert_eq!(fx.branch("zone/b"), moved_b);
    origin_unchanged(&fx, &origin);
}
