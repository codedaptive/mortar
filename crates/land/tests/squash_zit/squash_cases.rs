//! Cases 1, 3 and 5: a squash keeps origin's newer change and adds only the zone's; a conflict lists the files and
//! moves nothing; a zone at origin has nothing to squash.

use crate::fixture::{made, oid, refs, split_trailers, Fixture, ACTOR, SQUASH_REF, ZONE};
use land::{squash, SquashAnswer};
use std::path::PathBuf;

#[test]
fn a_squash_keeps_origins_newer_change_and_adds_only_the_zones() {
    let fx = Fixture::new();
    let o = fx.commit_on(&fx.base, &[("o", "origin\n")], "add o", None);
    fx.set_branch("main", &o);
    let z = fx.commit_on(&fx.base, &[("z", "zone\n")], "add z", Some("WO-a"));
    fx.set_branch("zone/land", &z);

    let squash = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    let commit = squash.commit.to_string();
    assert_eq!(fx.git(&["rev-list", "--parents", "-n", "1", &commit]), format!("{commit} {o}"));
    assert_eq!(fx.git(&["ls-tree", "--name-only", &commit]), "base\nf\no\nz");
    assert_eq!(fx.git(&["diff", "--name-only", &format!("{commit}^"), &commit]), "z");
    assert_eq!(fx.ref_of(SQUASH_REF), Some(commit.clone()));
    assert_eq!(fx.ref_of("refs/heads/zone/land"), Some(z.clone()));
    assert_eq!(fx.ref_of("refs/heads/main"), Some(o));
    assert_eq!(squash.source, oid(&z));
    assert_eq!(squash.requests, vec!["WO-a".to_string()]);
    let (text, trailers) = split_trailers(&fx.message(&commit));
    assert_eq!(text, "land: 1 requests\n\n- WO-a: add z\n");
    let expected = [
        format!("Mortar-Change-Id: {}", squash.change_id.as_str()),
        "Mortar-Zone: land".to_string(),
        format!("Mortar-Squash-Of: {z}"),
    ];
    assert_eq!(trailers, expected);
}

#[test]
fn a_conflict_lists_the_files_and_moves_nothing() {
    let fx = Fixture::new();
    let o = fx.commit_on(&fx.base, &[("f", "origin\n")], "origin f", None);
    fx.set_branch("main", &o);
    let z = fx.commit_on(&fx.base, &[("f", "zone\n")], "zone f", Some("WO-a"));
    fx.set_branch("zone/land", &z);

    let answer = squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap();

    assert_eq!(answer, SquashAnswer::Conflicted { files: vec![PathBuf::from("f")] });
    assert_eq!(fx.ref_of(SQUASH_REF), None);
    assert_eq!(fx.ref_of("refs/heads/zone/land"), Some(z));
    assert_eq!(fx.ref_of("refs/heads/main"), Some(o));
}

#[test]
fn a_zone_at_origin_has_nothing_to_squash() {
    let fx = Fixture::new();
    fx.set_branch("zone/land", &fx.base);

    let answer = squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap();

    assert_eq!(fx.ref_of(SQUASH_REF), None);
    assert_eq!(answer, SquashAnswer::Nothing { source: oid(&fx.base) });
}
