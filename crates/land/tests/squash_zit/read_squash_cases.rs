//! read_squash: a squash land::squash made reads back as the same Squash (requests in message order, none for a
//! zone without request commits); a commit missing Mortar-Squash-Of, one with a malformed Mortar-Change-Id, and one
//! whose request lines do not match its subject's count are refused naming the commit.

use crate::fixture::{made, oid, refs, Fixture, ACTOR, ZONE};
use land::{read_squash, squash, SquashError};

/// A well-formed change ID: 32 letters k..z.
const CHANGE_ID: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";

/// A commit on base B with base B's tree and exactly `message`, written with plain git.
fn commit_with_message(fx: &Fixture, message: &str) -> String {
    let tree = fx.git(&["rev-parse", &format!("{}^{{tree}}", fx.base)]);
    fx.git(&["commit-tree", &tree, "-p", &fx.base, "-m", message])
}

#[test]
fn read_squash_reads_back_the_squash_that_squash_made() {
    let fx = Fixture::new();
    let z1 = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("WO-b"));
    let z2 = fx.commit_on(&z1, &[("y", "y\n")], "add y", Some("WO-a"));
    let z3 = fx.commit_on(&z2, &[("x", "x\n")], "add x", Some("WO-b"));
    fx.set_branch("zone/land", &z3);
    let squash = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    let read = read_squash(&fx.land_git(), &squash.commit).unwrap();

    assert_eq!(read.requests, vec!["WO-b".to_string(), "WO-a".to_string()]);
    assert_eq!(read.source, oid(&z3));
    assert_eq!(read, squash);
}

#[test]
fn a_squash_of_a_zone_with_no_request_commits_reads_no_requests() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", None);
    fx.set_branch("zone/land", &z);
    let squash = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    let read = read_squash(&fx.land_git(), &squash.commit).unwrap();

    assert_eq!(read.requests, Vec::<String>::new());
    assert_eq!(read, squash);
}

#[test]
fn a_commit_without_mortar_squash_of_is_refused_naming_it() {
    let fx = Fixture::new();
    let commit = commit_with_message(
        &fx,
        &format!("land: 0 requests\n\nMortar-Change-Id: {CHANGE_ID}\nMortar-Zone: land\n"),
    );

    let error = read_squash(&fx.land_git(), &oid(&commit)).unwrap_err();

    let SquashError::TrailerMissing { commit: named, trailer } = &error else {
        panic!("expected TrailerMissing, got {error:?}")
    };
    assert_eq!(named, &oid(&commit));
    assert_eq!(trailer, "Mortar-Squash-Of");
    let text = error.to_string();
    assert!(text.contains("Mortar-Squash-Of"), "{text}");
    assert!(text.contains("squash the zone again"), "{text}");
}

#[test]
fn a_malformed_change_id_is_refused_naming_the_commit_and_the_trailer() {
    let fx = Fixture::new();
    let commit = commit_with_message(
        &fx,
        &format!("land: 0 requests\n\nMortar-Change-Id: not-a-change-id\nMortar-Squash-Of: {}\n", fx.base),
    );

    let error = read_squash(&fx.land_git(), &oid(&commit)).unwrap_err();

    let SquashError::TrailerMalformed { commit: named, trailer, value } = &error else {
        panic!("expected TrailerMalformed, got {error:?}")
    };
    assert_eq!(named, &oid(&commit));
    assert_eq!(trailer, "Mortar-Change-Id");
    assert_eq!(value, "not-a-change-id");
    let text = error.to_string();
    assert!(text.contains("Mortar-Change-Id"), "{text}");
    assert!(text.contains("squash the zone again"), "{text}");
}

#[test]
fn request_lines_that_do_not_match_the_count_are_refused() {
    let fx = Fixture::new();
    let commit = commit_with_message(
        &fx,
        &format!("land: 2 requests\n\n- WO-a: x\n\nMortar-Change-Id: {CHANGE_ID}\nMortar-Squash-Of: {}\n", fx.base),
    );

    let error = read_squash(&fx.land_git(), &oid(&commit)).unwrap_err();

    let SquashError::RequestsMalformed { commit: named } = &error else {
        panic!("expected RequestsMalformed, got {error:?}")
    };
    assert_eq!(named, &oid(&commit));
}
