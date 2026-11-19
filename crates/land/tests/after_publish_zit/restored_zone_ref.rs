//! A zone ref restored behind its replay record. Zone `land` is squashed at A1 and the squash kept;
//! after_publish(b, others=[land]) replays A1 onto b's origin as A1' and records A1 = A1' (fixture replayed). Zone
//! `land` is then set back to A1, the state an oplog restore of that one ref leaves: the record still names A1'.
//! Zone `c`, its own new file on the base, publishes; after_publish(c, others=[land]) replays A1 again, as A1'' on
//! c's origin, and must rewrite the record to A1'' (every clean replay writes its records). Then land's
//! kept squash publishes, and after_publish(land) takes A1'' through the record, puts the zone on origin and
//! deletes the record. Origin is never rewritten.

use crate::fixture::{
    land_and_publish, made, origin_untouched, record_ref, refs, replayed, zone, Fixture, ACTOR, PUBLISHED,
};
use land::{after_publish, ZoneMove};

/// Zone c's commit on the base: c.txt added, a.txt untouched, so it conflicts with nothing.
const C_OWN_FILES: [(&str, &str); 2] = [("a.txt", "one\ntwo\nthree\n"), ("c.txt", "c\n")];

#[test]
fn a_zone_ref_restored_behind_its_record_has_the_record_rewritten_by_the_next_replay() {
    let fx = Fixture::new();
    let replayed = replayed(&fx);
    let record = record_ref(PUBLISHED, &replayed.a1);
    assert_eq!(fx.git.read_ref(&record).unwrap(), Some(replayed.a1_replayed.clone()));
    fx.set_ref("refs/heads/zone/land", &replayed.a1);
    let c_tip = fx.join_onto("c", &fx.base, &C_OWN_FILES, "c work");
    let c_origin = land_and_publish(&fx, &made(&fx, "c"));
    let moves = after_publish(&fx.git, &refs(), &zone("c", vec![]), &[zone(PUBLISHED, vec![])], ACTOR).unwrap();
    assert_eq!(moves[0], ZoneMove::Moved { zone: "c".to_string(), old: c_tip, new: c_origin.clone(), rebase: vec![] });
    let ZoneMove::Moved { zone: name, old, new: a1_again, rebase } = &moves[1] else {
        panic!("zone land must be replayed onto c's origin: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &replayed.a1, &vec![]));
    assert_eq!(fx.git.commit_info(a1_again).unwrap().parents, vec![c_origin.clone()]);
    assert_eq!(fx.files(a1_again), ["a.txt", "b.txt", "c.txt", "p.txt"]);
    assert_eq!(fx.git.read_ref(&record).unwrap(), Some(a1_again.clone()));
    let origin = land_and_publish(&fx, &replayed.land_squash);
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![c_origin, replayed.land_squash.commit.clone()]);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected = vec![ZoneMove::Moved {
        zone: PUBLISHED.to_string(),
        old: a1_again.clone(),
        new: origin.clone(),
        rebase: vec![],
    }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    assert_eq!(fx.branch("zone/land"), origin);
    assert_eq!(fx.git.read_ref(&record).unwrap(), None);
    origin_untouched(&fx, &origin);
}
