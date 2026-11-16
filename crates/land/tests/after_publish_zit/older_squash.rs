//! An older squash published after a newer one of the same zone (S-121 (3)). Current: S1 at A1, then S2 at A2 on
//! A1; S2 publishes and the zone is put on its origin; then S1 lands, as a merge, and publishes (fixture
//! superseded). S1's source A1 is behind A2, the source of S2, which the zone already holds through origin, so
//! after_publish(land) answers Current, moves nothing and writes no op. Refused: S at A2 is made and kept, the zone
//! is set back to A1 and S1 made there, replacing the squash ref; S1 publishes and the zone is put on its origin;
//! then S lands and publishes. S's source A2 is not behind A1, the source of the squash the zone holds, so
//! after_publish(land) refuses with SquashOfNotInZone naming A2 and moves nothing. Origin is never rewritten.

use crate::fixture::{
    land_and_publish, made, origin_untouched, refs, superseded, zone, Fixture, ACTOR, A2_FILES, PUBLISHED,
    PUBLISHED_FILES,
};
use land::{after_publish, AfterPublishError, ZoneMove};

#[test]
fn an_older_squash_published_after_the_newer_one_leaves_the_zone_current_and_writes_no_op() {
    let fx = Fixture::new();
    let older = superseded(&fx);
    let records = fx.after_publish_records();
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected = vec![ZoneMove::Current { zone: PUBLISHED.to_string(), tip: older.s2_origin.clone() }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    assert_eq!(fx.branch("zone/land"), older.s2_origin);
    assert_eq!(fx.after_publish_records(), records);
    origin_untouched(&fx, &older.origin);
}

#[test]
fn a_squash_whose_source_is_not_behind_the_published_one_is_refused() {
    let fx = Fixture::new();
    let a1 = fx.join_onto(PUBLISHED, &fx.base, &PUBLISHED_FILES, "published work");
    let a2 = fx.join_onto(PUBLISHED, &a1, &A2_FILES, "second work");
    let squash_at_a2 = made(&fx, PUBLISHED);
    fx.set_ref("refs/heads/zone/land", &a1);
    let s1_origin = land_and_publish(&fx, &made(&fx, PUBLISHED));
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR).unwrap();
    assert_eq!(
        moves,
        vec![ZoneMove::Moved { zone: PUBLISHED.to_string(), old: a1, new: s1_origin.clone(), rebase: vec![] }]
    );
    let origin = land_and_publish(&fx, &squash_at_a2);
    assert_eq!(fx.git.commit_info(&origin).unwrap().parents, vec![s1_origin.clone(), squash_at_a2.commit.clone()]);
    let records = fx.after_publish_records();
    let moves = after_publish(&fx.git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    assert!(
        matches!(&moves, Err(AfterPublishError::SquashOfNotInZone { zone, squash_of })
            if zone == PUBLISHED && *squash_of == a2),
        "{moves:?}"
    );
    assert_eq!(fx.branch("zone/land"), s1_origin);
    assert_eq!(fx.after_publish_records(), records);
    origin_untouched(&fx, &origin);
}
