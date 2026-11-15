//! The publishing zone's squash source told apart by the replay that moved it, never by message, author and date.
//! The fixture gives every commit the same author and date, so commits with one subject share message, author and
//! date. Zone `land` is squashed and the squash kept; zone `b` is squashed, landed and published first, and
//! after_publish(b, others=[land]) replays zone `land` onto b's origin; then land's kept squash lands as a merge and
//! publishes. after_publish(land) must put the zone on origin when its tip is the replay of the squash source, also
//! when an earlier commit carries the same metadata or the source object was pruned, and must refuse with
//! SquashOfNotInZone, moving nothing, when the zone tip only shares the source's metadata.
//! On git's replay path, re-replaying an already published change collapses to nothing (the commit that becomes
//! empty is dropped), so the end state alone cannot show a wrong pick between two saves on every path; that case
//! also asserts the replay's record `refs/mortar/replayed/<zone>/<source>`.

use crate::fixture::{refs, zone, Fixture, ACTOR, B_FILES, PUBLISHED, PUBLISHED_FILES};
use gitio::{Oid, RunOpts};
use land::{
    after_publish, land, publish, squash, AfterPublishError, LandAnswer, PublishAnswer, Squash, SquashAnswer,
    ZoneMove,
};

/// The subject every zone `land` commit in these cases carries.
const SUBJECT: &str = "published work";
/// The later save X2 on X1: X1's files (PUBLISHED_FILES) plus x.txt.
const X2_FILES: [(&str, &str); 3] = [("a.txt", "one\nTWO\nthree\n"), ("p.txt", "published\n"), ("x.txt", "x\n")];
/// The re-save R on b's origin: the files of A1' (b.txt carried in by the replay) plus u.txt, work no squash holds.
const RESAVE_FILES: [(&str, &str); 4] =
    [("a.txt", "one\nTWO\nthree\n"), ("b.txt", "b\n"), ("p.txt", "published\n"), ("u.txt", "unsquashed\n")];

/// The zone's squash, made with land's own squash.
fn made(fx: &Fixture, zone: &str) -> Squash {
    let SquashAnswer::Made(made) = squash(&fx.git, &refs(), zone, ACTOR).unwrap() else {
        panic!("zone {zone} must squash");
    };
    made
}

/// The squash landed and published with land's own code; the origin publish answers.
fn land_and_publish(fx: &Fixture, made: &Squash) -> Oid {
    let LandAnswer::Landed { integration } = land(&fx.git, &refs(), made, ACTOR).unwrap() else {
        panic!("the squash must land");
    };
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: integration.clone() });
    integration
}

/// Zone `land` at `tip` squashed and the squash kept, b published, and after_publish(b, [land]) checked to move
/// zone `land` from `tip` to a replay on b's origin. Answers (land's kept squash, b's origin, the replayed tip).
fn replayed(fx: &Fixture, tip: &Oid) -> (Squash, Oid, Oid) {
    fx.join_onto("b", &fx.base, &B_FILES, "b work");
    let land_squash = made(fx, PUBLISHED);
    let b_origin = land_and_publish(fx, &made(fx, "b"));
    let moves = after_publish(&fx.git, &refs(), &zone("b", vec![]), &[zone(PUBLISHED, vec![])], ACTOR).unwrap();
    let ZoneMove::Moved { zone: name, old, new, rebase } = &moves[1] else {
        panic!("zone land must be replayed onto b's origin: {moves:?}");
    };
    assert_eq!((name.as_str(), old, rebase), (PUBLISHED, tip, &vec![]));
    assert_eq!(fx.git.commit_info(new).unwrap().message, format!("{SUBJECT}\n"));
    (land_squash, b_origin, new.clone())
}

/// main and the remote's main are both `origin`.
fn origin_untouched(fx: &Fixture, origin: &Oid) {
    assert_eq!(fx.branch("main"), *origin);
    assert_eq!(fx.remote_main(), Some(origin.clone()));
}

#[test]
fn two_saves_with_the_same_metadata_record_the_replay_of_the_later_one_and_publish_it() {
    let fx = Fixture::new();
    let x1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, SUBJECT);
    let x2 = fx.join_onto(PUBLISHED, &x1, &X2_FILES, SUBJECT);
    let (land_squash, b_origin, x2_replayed) = replayed(&fx, &x2);
    // The squash source X2 is recorded as replayed to X2', the zone tip, and so not to X1'.
    let recorded = fx.git.read_ref(&format!("refs/mortar/replayed/{PUBLISHED}/{}", x2.as_str()));
    assert_eq!(recorded.unwrap(), Some(x2_replayed.clone()));
    // Both saves were replayed, one for one: X2' on X1' on b's origin, X1' carrying X1's message, author and date.
    let x1_replayed = fx.git.commit_info(&x2_replayed).unwrap().parents[0].clone();
    assert_eq!(fx.git.commit_info(&x1_replayed).unwrap().parents, vec![b_origin.clone()]);
    let (x1_info, x1_replayed_info) = (fx.git.commit_info(&x1).unwrap(), fx.git.commit_info(&x1_replayed).unwrap());
    assert_eq!(x1_replayed_info.message, x1_info.message);
    assert_eq!(x1_replayed_info.message, fx.git.commit_info(&x2).unwrap().message);
    let origin = land_and_publish(&fx, &land_squash);
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![b_origin, land_squash.commit.clone()]);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected =
        vec![ZoneMove::Moved { zone: PUBLISHED.to_string(), old: x2_replayed, new: origin.clone(), rebase: vec![] }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    assert_eq!(moves.unwrap(), expected);
    assert_eq!(fx.branch("zone/land"), origin);
    origin_untouched(&fx, &origin);
}

#[test]
fn a_re_save_with_the_same_metadata_after_the_replay_is_refused_not_dropped() {
    let fx = Fixture::new();
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, SUBJECT);
    let (land_squash, b_origin, a1_replayed) = replayed(&fx, &a1);
    assert_eq!(fx.git.commit_info(&a1_replayed).unwrap().parents, vec![b_origin.clone()]);
    // The zone undone to b's origin and saved again: same subject as A1, and u.txt, which no squash holds.
    let resaved = fx.join_onto(PUBLISHED, &b_origin, &RESAVE_FILES, SUBJECT);
    assert_eq!(fx.git.commit_info(&resaved).unwrap().message, fx.git.commit_info(&a1).unwrap().message);
    let origin = land_and_publish(&fx, &land_squash);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    assert!(
        matches!(&moves, Err(AfterPublishError::SquashOfNotInZone { zone, squash_of })
            if zone == PUBLISHED && *squash_of == a1),
        "{moves:?}"
    );
    assert_eq!(fx.branch("zone/land"), resaved);
    assert_eq!(fx.file(&resaved, "u.txt"), "unsquashed\n");
    origin_untouched(&fx, &origin);
}

#[test]
fn the_publishing_zone_moves_when_its_squash_source_was_pruned() {
    let fx = Fixture::new();
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, SUBJECT);
    let (land_squash, _, a1_replayed) = replayed(&fx, &a1);
    // No ref holds A1 now (zone/land is A1', the squash sits on the base): expire the reflogs and prune it.
    for args in [&["reflog", "expire", "--expire=now", "--all"][..], &["gc", "--prune=now", "--quiet"][..]] {
        fx.git.run(args, &RunOpts::default()).unwrap();
    }
    let holders = fx.git.run(&["for-each-ref", "--contains", a1.as_str()], &RunOpts::default());
    assert_eq!(fx.git.rev_parse(a1.as_str()).unwrap(), None, "A1 still held: {holders:?}");
    let origin = land_and_publish(&fx, &land_squash);
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected =
        vec![ZoneMove::Moved { zone: PUBLISHED.to_string(), old: a1_replayed, new: origin.clone(), rebase: vec![] }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    assert_eq!(moves.unwrap(), expected);
    assert_eq!(fx.branch("zone/land"), origin);
    origin_untouched(&fx, &origin);
}
