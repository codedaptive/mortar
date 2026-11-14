//! source(): the wrapped error for Squash, Land, Git, Oplog and State; none for the refusals publish makes itself.

use super::{LandError, LandingStateError, PublishError};
use crate::squash::SquashError;
use gitio::{GitError, Oid, PushRefusal};
use oplog::OplogError;
use std::error::Error;
use std::path::PathBuf;

fn git_error() -> GitError {
    GitError { args: vec!["push".to_string()], code: Some(1), stderr: "bad".to_string() }
}

#[test]
fn squash_source_is_the_inner_squash_error() {
    let error = PublishError::Squash(SquashError::RefMissing { refname: "refs/heads/main".to_string() });
    let refname = error.source().and_then(|s| s.downcast_ref::<SquashError>()).map(|e| match e {
        SquashError::RefMissing { refname } => refname.as_str(),
        _ => "another variant",
    });
    assert_eq!(refname, Some("refs/heads/main"));
}

#[test]
fn land_source_is_the_inner_land_error() {
    let commit = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    let error = PublishError::Land(LandError::NoZone { commit });
    let commit = error.source().and_then(|s| s.downcast_ref::<LandError>()).map(|e| match e {
        LandError::NoZone { commit } => commit.as_str(),
        _ => "another variant",
    });
    assert_eq!(commit, Some("1111111111111111111111111111111111111111"));
}

#[test]
fn git_source_is_the_inner_git_error() {
    let error = PublishError::Git(git_error());
    assert_eq!(error.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&git_error()));
}

#[test]
fn oplog_source_is_the_inner_oplog_error() {
    let error = PublishError::Oplog(OplogError::UnknownOp { op_id: "op-3".to_string() });
    let op_id = error.source().and_then(|s| s.downcast_ref::<OplogError>()).map(|e| match e {
        OplogError::UnknownOp { op_id } => op_id.as_str(),
        _ => "another variant",
    });
    assert_eq!(op_id, Some("op-3"));
}

#[test]
fn state_source_is_the_inner_landing_state_error() {
    let inner = LandingStateError::Corrupt { path: PathBuf::from("/p"), reason: "why".to_string() };
    let error = PublishError::State(inner);
    let reason = error.source().and_then(|s| s.downcast_ref::<LandingStateError>()).map(|e| match e {
        LandingStateError::Corrupt { reason, .. } => reason.as_str(),
        _ => "another variant",
    });
    assert_eq!(reason, Some("why"));
}

#[test]
fn refusals_publish_makes_itself_have_no_source() {
    let refusals = [
        PublishError::PushRefused { refname: "refs/heads/main".to_string(), refusal: PushRefusal::StaleLease },
        PublishError::RemoteMissing { remote: "origin".to_string(), refname: "refs/heads/main".to_string() },
    ];
    for refusal in &refusals {
        assert!(refusal.source().is_none(), "{refusal:?}");
    }
}
