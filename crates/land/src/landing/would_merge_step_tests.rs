//! would_merge_step: a Commit target continues with that commit; a Conflicted target stops naming the step, the
//! squash's commit and the files in order.

use super::{would_merge_step, WouldMergeWith};
use crate::landing::LandingTarget;
use gitio::Oid;
use std::ops::ControlFlow;
use std::path::PathBuf;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn step_answers() {
    let squash = "1111111111111111111111111111111111111111";
    let merged = "2222222222222222222222222222222222222222";
    let rows: [(&str, usize, LandingTarget, ControlFlow<WouldMergeWith, Oid>); 2] = [
        ("commit continues", 1, LandingTarget::Commit(oid(merged)), ControlFlow::Continue(oid(merged))),
        (
            "conflicted at step 2 breaks",
            2,
            LandingTarget::Conflicted(vec![PathBuf::from("src/b.rs"), PathBuf::from("a.txt")]),
            ControlFlow::Break(WouldMergeWith::Conflicted {
                step: 2,
                commit: oid(squash),
                files: vec![PathBuf::from("src/b.rs"), PathBuf::from("a.txt")],
            }),
        ),
    ];
    for (name, step, target, expected) in rows {
        assert_eq!(would_merge_step(step, &oid(squash), target), expected, "{name}");
    }
}
