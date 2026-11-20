//! would_merge says Yes when the squash merges into integration and No with the conflicted files; neither
//! moves the integration branch.

use crate::fixture::{made, refs, Fixture, ACTOR, ZONE};
use land::{squash, would_merge, WouldMerge};
use std::path::PathBuf;

#[test]
fn would_merge_says_yes_when_integration_is_origin() {
    let fx = Fixture::new();
    let z = fx.commit_on(&fx.base, &[("f", "zone\n")], "zone f", Some("r1"));
    fx.set_branch("zone/land", &z);
    let squash = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    let answer = would_merge(&fx.land_git(), &refs(), &squash).unwrap();

    assert_eq!(fx.ref_of("refs/heads/integration"), Some(fx.base.clone()));
    assert_eq!(answer, WouldMerge::Yes);
}

#[test]
fn would_merge_says_no_with_the_conflicted_files() {
    let fx = Fixture::new();
    let i = fx.commit_on(&fx.base, &[("f", "integration\n")], "integration f", None);
    fx.set_branch("integration", &i);
    let z = fx.commit_on(&fx.base, &[("f", "zone\n")], "zone f", Some("r1"));
    fx.set_branch("zone/land", &z);
    let squash = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    let answer = would_merge(&fx.land_git(), &refs(), &squash).unwrap();

    assert_eq!(fx.ref_of("refs/heads/integration"), Some(i));
    assert_eq!(answer, WouldMerge::No { files: vec![PathBuf::from("f")] });
}
