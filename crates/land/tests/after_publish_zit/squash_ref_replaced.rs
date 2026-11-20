//! An older squash published after a newer one replaced the squash ref. Zone `land` is squashed at A1 (S1,
//! kept), then A2 joins on A1 and S2 is made at A2, replacing `refs/mortar/squash/land`. Zone `b` publishes and
//! after_publish(b, others=[land]) replays A1 and A2 onto b's origin, recording both, not only the squash ref's
//! source (fixture replayed_chain). Then S1 lands, as a merge, and publishes: after_publish(land) takes
//! A1' through A1's record, replays only A2' onto origin, deletes A1's record (the published source) and carries
//! A2's record to A2's new image, the zone tip. Origin is never rewritten.

use crate::fixture::{
    land_and_publish, made, origin_untouched, record_ref, refs, replayed_chain, zone, Fixture, ACTOR, A2_FILES,
    PUBLISHED, PUBLISHED_FILES,
};
use land::{after_publish, ZoneMove};

#[test]
fn an_older_squash_published_after_a_newer_one_replaced_the_squash_ref_moves_the_zone() {
    let fx = Fixture::new();
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let s1 = made(&fx, PUBLISHED);
    let a2 = fx.join_onto(PUBLISHED, &a1, &A2_FILES, "second work");
    made(&fx, PUBLISHED);
    let chain = replayed_chain(&fx, &a1, &a2);
    let origin = land_and_publish(&fx, &s1);
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![chain.b_origin, s1.commit.clone()]);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    assert!(moves.is_ok(), "{moves:?}");
    let moves = moves.unwrap();
    let [ZoneMove::Moved { zone: name, old, new, rebase }] = moves.as_slice() else {
        panic!("the published zone must move: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, &chain.a2_replayed, &vec![]));
    assert_eq!(fx.branch("zone/land"), *new);
    // Only A2' is replayed: A1' stands for S1's source, which origin holds through S1.
    assert_eq!(fx.git.commit_info(new).unwrap().parents, vec![origin.clone()]);
    assert_eq!(fx.files(new), ["a.txt", "b.txt", "p.txt", "q.txt"]);
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, &a1)).unwrap(), None);
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, &a2)).unwrap(), Some(new.clone()));
    assert_eq!(fx.git.read_ref(&record_ref(PUBLISHED, &chain.a2_replayed)).unwrap(), Some(new.clone()));
    origin_untouched(&fx, &origin);
}
