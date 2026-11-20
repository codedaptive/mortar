//! A re-squash keeps the change ID; a repeat answers Same and writes nothing; once the prior squash is in
//! origin, the next squash draws a new change ID.

use crate::fixture::{made, oid, refs, Fixture, ACTOR, SQUASH_REF, ZONE};
use land::{squash, SquashAnswer};

#[test]
fn a_resquash_keeps_the_change_id() {
    let fx = Fixture::new();
    let z1 = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z1);
    let first = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());
    let z2 = fx.commit_on(&z1, &[("y", "more\n")], "add y", Some("r2"));
    fx.set_branch("zone/land", &z2);

    let second = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    assert_ne!(second.commit, first.commit);
    assert_eq!(second.source, oid(&z2));
    assert_eq!(fx.ref_of(SQUASH_REF), Some(second.commit.to_string()));
    assert_eq!(second.change_id, first.change_id);
}

#[test]
fn a_repeated_squash_answers_same_and_writes_nothing() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z);
    let first = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());
    let log = fx.oplog();

    let again = squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap();

    assert_eq!(again, SquashAnswer::Same(first.clone()));
    assert_eq!(fx.ref_of(SQUASH_REF), Some(first.commit.to_string()));
    assert!(log.is_some());
    assert_eq!(fx.oplog(), log);
}

#[test]
fn a_squash_after_the_prior_one_is_in_origin_draws_a_new_change_id() {
    let fx = Fixture::new();
    let z1 = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z1);
    let first = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());
    fx.set_branch("main", &first.commit.to_string());
    let z2 = fx.commit_on(&z1, &[("y", "more\n")], "add y", Some("r1"));
    fx.set_branch("zone/land", &z2);

    let next = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    assert_eq!(fx.git(&["rev-parse", &format!("{}^", next.commit)]), first.commit.to_string());
    assert_eq!(fx.ref_of(SQUASH_REF), Some(next.commit.to_string()));
    assert_ne!(next.change_id, first.change_id);
}
