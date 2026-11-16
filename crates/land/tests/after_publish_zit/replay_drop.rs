//! A zone whose replay drops a commit that becomes empty still moves, and records the dropped commit as the commit
//! it collapsed into (S-121 W). Zone `land` holds A1 (d.txt added) then A2 (PUBLISHED_FILES' changes), and its
//! squash, source A2, is kept; zone `b` adds the same d.txt with b.txt, so on b's origin A1 replays to nothing and
//! is dropped. after_publish(b, others=[land]) moves zone `land` to A2' on b's origin and records A1 = b's origin
//! and A2 = A2'; when land's own squash publishes, after_publish(land) puts the zone on origin and deletes both
//! records. Run once with git's replay probe as installed and once with it forced unavailable: the answer must not
//! depend on the installation (an installation without native replay runs the forced arm only). Origin is never
//! rewritten.

use crate::fixture::{
    land_and_publish, made, origin_untouched, record_ref, refs, zone, Fixture, ACTOR, PUBLISHED,
};
use crate::shims::runner;
use land::{after_publish, ZoneMove};

/// A1: the base files plus d.txt, the content zone b also adds.
const A1_FILES: [(&str, &str); 2] = [("a.txt", "one\ntwo\nthree\n"), ("d.txt", "dup\n")];
/// A2 on A1: PUBLISHED_FILES' changes (a.txt's middle line, p.txt) with A1's d.txt.
const A2_FILES: [(&str, &str); 3] = [("a.txt", "one\nTWO\nthree\n"), ("d.txt", "dup\n"), ("p.txt", "published\n")];
/// Zone b's commit on the base: B_FILES plus the same d.txt as A1, so A1 is empty on b's origin.
const B_DUP_FILES: [(&str, &str); 3] = [("a.txt", "one\ntwo\nthree\n"), ("b.txt", "b\n"), ("d.txt", "dup\n")];

#[test]
fn a_zone_whose_replay_drops_a_commit_moves_records_it_as_its_onto_and_its_publish_deletes_both_records() {
    for fallback in [false, true] {
        let fx = Fixture::new();
        let Some(git) = runner(&fx, fallback) else { continue };
        let a1 = fx.join_onto(PUBLISHED, &fx.base, &A1_FILES, "duplicated work");
        let a2 = fx.join_onto(PUBLISHED, &a1, &A2_FILES, "published work");
        let land_squash = made(&fx, PUBLISHED);
        let b_tip = fx.join_onto("b", &fx.base, &B_DUP_FILES, "b work");
        let b_origin = land_and_publish(&fx, &made(&fx, "b"));
        let moves = after_publish(&git, &refs(), &zone("b", vec![]), &[zone(PUBLISHED, vec![])], ACTOR);
        assert!(moves.is_ok(), "fallback={fallback}: {moves:?}");
        let moves = moves.unwrap();
        let b_moved = ZoneMove::Moved { zone: "b".to_string(), old: b_tip, new: b_origin.clone(), rebase: vec![] };
        assert_eq!(moves[0], b_moved, "fallback={fallback}");
        let ZoneMove::Moved { zone: name, old, new: a2_replayed, rebase } = &moves[1] else {
            panic!("fallback={fallback}: zone land must move onto b's origin: {moves:?}");
        };
        assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &a2, &vec![]), "fallback={fallback}");
        // A1 dropped: A2' sits directly on b's origin and holds b's files, d.txt once, and A2's changes.
        assert_eq!(fx.git.commit_info(a2_replayed).unwrap().parents, vec![b_origin.clone()], "fallback={fallback}");
        assert_eq!(fx.files(a2_replayed), ["a.txt", "b.txt", "d.txt", "p.txt"], "fallback={fallback}");
        assert_eq!(fx.branch("zone/land"), *a2_replayed, "fallback={fallback}");
        let (a1_record, a2_record) = (record_ref(PUBLISHED, &a1), record_ref(PUBLISHED, &a2));
        assert_eq!(fx.git.read_ref(&a1_record).unwrap(), Some(b_origin.clone()), "fallback={fallback}");
        assert_eq!(fx.git.read_ref(&a2_record).unwrap(), Some(a2_replayed.clone()), "fallback={fallback}");
        let origin = land_and_publish(&fx, &land_squash);
        assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![b_origin, land_squash.commit.clone()]);
        let moves = after_publish(&git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
        let expected = vec![ZoneMove::Moved {
            zone: PUBLISHED.to_string(),
            old: a2_replayed.clone(),
            new: origin.clone(),
            rebase: vec![],
        }];
        assert!(matches!(&moves, Ok(answer) if *answer == expected), "fallback={fallback}: {moves:?}");
        assert_eq!(fx.branch("zone/land"), origin, "fallback={fallback}");
        assert_eq!(fx.git.read_ref(&a1_record).unwrap(), None, "fallback={fallback}");
        assert_eq!(fx.git.read_ref(&a2_record).unwrap(), None, "fallback={fallback}");
        origin_untouched(&fx, &origin);
    }
}
