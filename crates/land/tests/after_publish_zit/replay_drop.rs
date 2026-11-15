//! A zone whose replay would drop a commit while it owes a replay record is left in place (S-120). Zone `land`
//! holds A1 (d.txt added) then A2 (PUBLISHED_FILES' changes), and its squash, source A2, is kept; zone `b` adds the
//! same d.txt with b.txt, so on b's origin A1 replays to nothing and git drops it. after_publish(b, others=[land])
//! must list zone `land` as ReplayDropped naming A2, leave it at A2 and write no record for A2; when land's own
//! squash publishes, after_publish(land) puts the zone on origin by the normal path. Origin is never rewritten.

use crate::fixture::{land_and_publish, made, origin_untouched, refs, zone, Fixture, ACTOR, PUBLISHED};
use land::{after_publish, ZoneMove};

/// A1: the base files plus d.txt, the content zone b also adds.
const A1_FILES: [(&str, &str); 2] = [("a.txt", "one\ntwo\nthree\n"), ("d.txt", "dup\n")];
/// A2 on A1: PUBLISHED_FILES' changes (a.txt's middle line, p.txt) with A1's d.txt.
const A2_FILES: [(&str, &str); 3] = [("a.txt", "one\nTWO\nthree\n"), ("d.txt", "dup\n"), ("p.txt", "published\n")];
/// Zone b's commit on the base: B_FILES plus the same d.txt as A1, so A1 is empty on b's origin.
const B_DUP_FILES: [(&str, &str); 3] = [("a.txt", "one\ntwo\nthree\n"), ("b.txt", "b\n"), ("d.txt", "dup\n")];

#[test]
fn a_zone_whose_replay_dropped_a_commit_owing_a_record_stays_put_and_moves_at_its_own_publish() {
    let fx = Fixture::new();
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &A1_FILES, "duplicated work");
    let a2 = fx.join_onto(PUBLISHED, &a1, &A2_FILES, "published work");
    let land_squash = made(&fx, PUBLISHED);
    let b_tip = fx.join_onto("b", &fx.base, &B_DUP_FILES, "b work");
    let b_origin = land_and_publish(&fx, &made(&fx, "b"));
    let moves = after_publish(&fx.git, &refs(), &zone("b", vec![]), &[zone(PUBLISHED, vec![])], ACTOR);
    assert!(moves.is_ok(), "{moves:?}");
    let moves = moves.unwrap();
    assert_eq!(
        moves,
        vec![
            ZoneMove::Moved { zone: "b".to_string(), old: b_tip, new: b_origin.clone(), rebase: vec![] },
            ZoneMove::ReplayDropped { zone: PUBLISHED.to_string(), tip: a2.clone(), squash_of: a2.clone() },
        ]
    );
    assert_eq!(fx.branch("zone/land"), a2);
    let record = format!("refs/mortar/replayed/{PUBLISHED}/{}", a2.as_str());
    assert_eq!(fx.git.read_ref(&record).unwrap(), None);
    let origin = land_and_publish(&fx, &land_squash);
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![b_origin, land_squash.commit.clone()]);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected = vec![ZoneMove::Moved { zone: PUBLISHED.to_string(), old: a2, new: origin.clone(), rebase: vec![] }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    assert_eq!(fx.branch("zone/land"), origin);
    origin_untouched(&fx, &origin);
}
