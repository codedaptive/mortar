//! Ordered squash previews use successive results without changing refs or reachable objects.

use crate::fixture::{made, oid, refs, Fixture, ACTOR, ZONE};
use land::{squash, would_merge_with, Squash, WouldMergeWith};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn squashes(fx: &Fixture, second_file: &str) -> (Squash, Squash) {
    let first = fx.commit_on(&fx.base, &[("f", "first\n")], "first", Some("r1"));
    fx.set_branch("zone/land", &first);
    let first = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    let second = fx.commit_on(&fx.base, &[(second_file, "second\n")], "second", Some("r2"));
    fx.set_branch("zone/join", &second);
    let second = made(squash(&fx.land_git(), &refs(), "join", ACTOR).unwrap());
    (first, second)
}

#[test]
fn ordered_squashes_merge_cleanly() {
    let fx = Fixture::new();
    let (first, second) = squashes(&fx, "other");

    assert_eq!(would_merge_with(&fx.land_git(), &refs(), &first, &[second]).unwrap(), WouldMergeWith::Clean);
}

#[test]
fn second_squash_conflicts_with_first_result() {
    let fx = Fixture::new();
    let (first, second) = squashes(&fx, "f");
    let second_commit = oid(&fx.git(&["rev-parse", "refs/mortar/squash/join"]));

    assert_eq!(
        would_merge_with(&fx.land_git(), &refs(), &first, &[second]).unwrap(),
        WouldMergeWith::Conflicted { step: 2, commit: second_commit, files: vec![PathBuf::from("f")] }
    );
}

#[test]
fn preview_leaves_refs_and_reachable_objects_unchanged() {
    let fx = Fixture::new();
    let (first, second) = squashes(&fx, "other");
    let refs_before = fx.raw(&["for-each-ref", "--format=%(refname) %(objectname)"]);
    let objects_before: BTreeSet<_> = fx.raw(&["rev-list", "--objects", "--all"]).lines().map(str::to_owned).collect();

    assert_eq!(would_merge_with(&fx.land_git(), &refs(), &first, &[second]).unwrap(), WouldMergeWith::Clean);

    let objects_after: BTreeSet<_> = fx.raw(&["rev-list", "--objects", "--all"]).lines().map(str::to_owned).collect();
    assert_eq!(fx.raw(&["for-each-ref", "--format=%(refname) %(objectname)"]), refs_before);
    assert_eq!(objects_after, objects_before);
    assert_eq!(fx.ref_of("refs/heads/integration"), Some(fx.base.clone()));
}
