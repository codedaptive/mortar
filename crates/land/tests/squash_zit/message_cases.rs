//! squash_with_message: the caller's text, trimmed, comes before Mortar's request lines and trailers and reads back
//! with read_squash; the same message again answers Same and writes nothing; a new message on an unchanged zone is a
//! new commit with the same tree, parent and change ID.

use crate::fixture::{made, oid, refs, Fixture, ACTOR, SQUASH_REF, ZONE};
use land::{read_squash, squash, squash_with_message, SquashAnswer};

#[test]
fn a_caller_message_comes_before_the_request_lines_and_reads_back() {
    let fx = Fixture::new();
    let z1 = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r2"));
    let z2 = fx.commit_on(&z1, &[("y", "y\n")], "add y", Some("r1"));
    let z3 = fx.commit_on(&z2, &[("x", "x\n")], "add x", Some("r2"));
    fx.set_branch("zone/land", &z3);

    let made_squash = made(
        squash_with_message(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard\n\nWhy it was wrong.\n"))
            .unwrap(),
    );

    let expected = format!(
        "Fix the guard\n\nWhy it was wrong.\n\n- r2: add z; add x\n- r1: add y\n\n\
         Mortar-Change-Id: {}\nMortar-Zone: land\nMortar-Squash-Of: {z3}\n",
        made_squash.change_id.as_str()
    );
    assert_eq!(fx.message(&made_squash.commit.to_string()), expected);
    let read = read_squash(&fx.land_git(), &made_squash.commit).unwrap();
    assert_eq!(read.requests, vec!["r2".to_string(), "r1".to_string()]);
    assert_eq!(read.source, oid(&z3));
    assert_eq!(read, made_squash);
}

#[test]
fn the_same_message_again_answers_same_and_writes_nothing() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z);
    let first = made(squash_with_message(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")).unwrap());
    let log = fx.oplog();

    let again = squash_with_message(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Fix the guard")).unwrap();

    assert_eq!(again, SquashAnswer::Same(first.clone()));
    assert_eq!(fx.ref_of(SQUASH_REF), Some(first.commit.to_string()));
    assert!(log.is_some());
    assert_eq!(fx.oplog(), log);
}

#[test]
fn a_new_message_on_an_unchanged_squash_is_a_new_commit_with_the_same_change_id() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("r1"));
    fx.set_branch("zone/land", &z);
    let first = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    let relabelled =
        made(squash_with_message(&fx.land_git(), &refs(), ZONE, ACTOR, Some("Relabel the squash")).unwrap());

    let (old, new) = (first.commit.to_string(), relabelled.commit.to_string());
    assert_ne!(new, old);
    assert_eq!(fx.git(&["rev-parse", &format!("{new}^{{tree}}")]), fx.git(&["rev-parse", &format!("{old}^{{tree}}")]));
    assert_eq!(fx.git(&["rev-list", "--parents", "-n", "1", &new]), format!("{new} {}", fx.base));
    assert_eq!(fx.git(&["rev-list", "--parents", "-n", "1", &old]), format!("{old} {}", fx.base));
    assert_eq!(relabelled.change_id, first.change_id);
    assert_eq!(fx.ref_of(SQUASH_REF), Some(new.clone()));
    assert_eq!(fx.message(&new).lines().next(), Some("Relabel the squash"));
}
