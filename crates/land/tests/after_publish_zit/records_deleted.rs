//! The publish deletes every replay record of its zone at or behind its source, in the op that moves the zone, and
//! in a records-only op when the zone is Current (S-121 D). Replay: zone `land` holds A1 then A2, squashed at A2
//! and kept; after_publish(b, others=[land]) replays both onto b's origin and records A1 = A1', A2 = A2' (fixture
//! replayed_chain); A3 joins on A2'; the squash lands as a merge and publishes; after_publish(land) replays only A3
//! onto origin and, in one done op, moves the zone, deletes A2's record (the published source) and A1's (A1' is
//! behind A2') and writes A3's. Current: an older squash S1 at A1 published after the newer S2 at A2 (fixture
//! superseded) leaves the zone Current, and a stale record keyed by A1, naming a commit the zone does not hold, is
//! deleted by one records-only op while A2's record, neither keyed by A1 nor behind it, stays.

use crate::fixture::{
    deleted, done_moves, land_and_publish, made, origin_untouched, record_ref, refs, replayed_chain, superseded, zone,
    zone_move, Fixture, ACTOR, A2_FILES, PUBLISHED, PUBLISHED_FILES,
};
use land::{after_publish, ZoneMove};
use oplog::RefMove;

/// A3 on A2': the files of A2' (b.txt carried in by the replay) plus j.txt.
const A3_FILES: [(&str, &str); 5] = [
    ("a.txt", "one\nTWO\nthree\n"),
    ("b.txt", "b\n"),
    ("j.txt", "joined\n"),
    ("p.txt", "published\n"),
    ("q.txt", "second\n"),
];

/// `moves` with its record moves (all but the first, the zone's) in refname order: the op's order among them is
/// not part of the contract.
fn by_refname(mut moves: Vec<RefMove>) -> Vec<RefMove> {
    moves[1..].sort_by(|left, right| left.refname.cmp(&right.refname));
    moves
}

#[test]
fn a_replaying_publish_deletes_the_records_at_or_behind_its_source_and_writes_its_own_in_one_op() {
    let fx = Fixture::new();
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let a2 = fx.join_onto(PUBLISHED, &a1, &A2_FILES, "second work");
    let land_squash = made(&fx, PUBLISHED);
    let chain = replayed_chain(&fx, &a1, &a2);
    let a3 = fx.join_onto(PUBLISHED, &chain.a2_replayed, &A3_FILES, "joined after the replay");
    let origin = land_and_publish(&fx, &land_squash);
    let before = done_moves(&fx);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    assert!(moves.is_ok(), "{moves:?}");
    let moves = moves.unwrap();
    let [ZoneMove::Moved { zone: name, old, new, rebase }] = moves.as_slice() else {
        panic!("the published zone must move: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &a3, &vec![]));
    assert_eq!(fx.git.commit_info(new).unwrap().parents, vec![origin.clone()]);
    assert_eq!(fx.files(new), ["a.txt", "b.txt", "j.txt", "p.txt", "q.txt"]);
    let expected = [
        zone_move("refs/heads/zone/land", &a3, new),
        vec![
            deleted(&record_ref(PUBLISHED, &a1), &chain.a1_replayed),
            deleted(&record_ref(PUBLISHED, &a2), &chain.a2_replayed),
            RefMove { refname: record_ref(PUBLISHED, &a3), old: None, new: new.clone() },
        ],
    ]
    .concat();
    let after = done_moves(&fx);
    assert_eq!(after.len(), before.len() + 1, "{after:?}");
    assert_eq!(by_refname(after[before.len()].clone()), by_refname(expected));
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, &a1)).unwrap(), None);
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, &a2)).unwrap(), None);
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, &a3)).unwrap(), Some(new.clone()));
    origin_untouched(&fx, &origin);
}

#[test]
fn a_current_publish_deletes_only_the_stale_record_of_its_source_in_a_records_only_op() {
    let fx = Fixture::new();
    let older = superseded(&fx);
    let (a1_record, a2_record) = (record_ref(PUBLISHED, &older.a1), record_ref(PUBLISHED, &older.a2));
    // Both name A2, which the zone (on S2's origin, a squash on the base) does not hold.
    fx.set_ref(&a1_record, &older.a2);
    fx.set_ref(&a2_record, &older.a2);
    let before = done_moves(&fx);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected = vec![ZoneMove::Current { zone: PUBLISHED.to_string(), tip: older.s2_origin.clone() }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    assert_eq!(fx.branch("zone/land"), older.s2_origin);
    assert_eq!(done_moves(&fx), [before, vec![vec![deleted(&a1_record, &older.a2)]]].concat());
    assert_eq!(fx.git.read_ref(&a1_record).unwrap(), None);
    assert_eq!(fx.git.read_ref(&a2_record).unwrap(), Some(older.a2.clone()));
    origin_untouched(&fx, &older.origin);
}
