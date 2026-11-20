//! source(): the wrapped error for Git, Squash, Join and Land; none for the refusals after_publish makes itself
//! (Unrelated, NoSquashOf, SquashOfNotInZone).

use super::AfterPublishError;
use crate::landing::LandError;
use crate::squash::SquashError;
use gitio::{GitError, Oid};
use join::JoinError;
use std::error::Error;

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn git_error() -> GitError {
    GitError { args: vec!["merge-base".to_string()], code: Some(1), stderr: "bad".to_string() }
}

#[test]
fn git_source_is_the_inner_git_error() {
    let error = AfterPublishError::Git(git_error());
    assert_eq!(error.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&git_error()));
}

#[test]
fn squash_source_is_the_inner_squash_error() {
    let error = AfterPublishError::Squash(SquashError::RefMissing { refname: "refs/heads/zone/land".to_string() });
    let refname = error.source().and_then(|s| s.downcast_ref::<SquashError>()).map(|e| match e {
        SquashError::RefMissing { refname } => refname.as_str(),
        _ => "another variant",
    });
    assert_eq!(refname, Some("refs/heads/zone/land"));
}

#[test]
fn join_source_is_the_inner_join_error() {
    let inner = JoinError::TipMissing("refs/heads/zone/land".to_string());
    let error = AfterPublishError::Join(inner.clone());
    assert_eq!(error.source().and_then(|s| s.downcast_ref::<JoinError>()), Some(&inner));
}

#[test]
fn land_source_is_the_inner_land_error() {
    let error = AfterPublishError::Land(LandError::NoZone { commit: oid("1111111111111111111111111111111111111111") });
    let commit = error.source().and_then(|s| s.downcast_ref::<LandError>()).map(|e| match e {
        LandError::NoZone { commit } => commit.as_str(),
        _ => "another variant",
    });
    assert_eq!(commit, Some("1111111111111111111111111111111111111111"));
}

#[test]
fn refusals_after_publish_makes_itself_have_no_source() {
    let refusals = [
        AfterPublishError::Unrelated {
            zone: "land".to_string(),
            origin: oid("1111111111111111111111111111111111111111"),
            tip: oid("2222222222222222222222222222222222222222"),
        },
        AfterPublishError::NoSquashOf { zone: "land".to_string() },
        AfterPublishError::SquashOfNotInZone {
            zone: "land".to_string(),
            squash_of: oid("3333333333333333333333333333333333333333"),
        },
    ];
    for refusal in &refusals {
        assert!(refusal.source().is_none(), "{refusal:?}");
    }
}
