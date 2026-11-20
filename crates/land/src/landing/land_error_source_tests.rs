//! source(): the wrapped error for Squash, Git, Oplog, Trailers and State; none for the refusals land and back_out
//! make themselves.

use super::{LandError, LandingStateError};
use crate::squash::SquashError;
use gitio::{GitError, Oid};
use oplog::{MortarTrailerError, OpPhase, OpRecord, OplogError};
use std::error::Error;
use std::path::PathBuf;

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn git_error() -> GitError {
    GitError { args: vec!["merge-tree".to_string()], code: Some(1), stderr: "bad".to_string() }
}

fn record() -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "k".to_string(),
        kind: "land".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 1,
        phase: OpPhase::Failed,
        moves: Vec::new(),
        result: None,
    }
}

#[test]
fn squash_source_is_the_inner_squash_error() {
    let error = LandError::Squash(SquashError::RefMissing { refname: "refs/heads/main".to_string() });
    let refname = error.source().and_then(|s| s.downcast_ref::<SquashError>()).map(|e| match e {
        SquashError::RefMissing { refname } => refname.as_str(),
        _ => "another variant",
    });
    assert_eq!(refname, Some("refs/heads/main"));
}

#[test]
fn git_source_is_the_inner_git_error() {
    let error = LandError::Git(git_error());
    assert_eq!(error.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&git_error()));
}

#[test]
fn oplog_source_is_the_inner_oplog_error() {
    let error = LandError::Oplog(OplogError::UnknownOp { op_id: "op-3".to_string() });
    let op_id = error.source().and_then(|s| s.downcast_ref::<OplogError>()).map(|e| match e {
        OplogError::UnknownOp { op_id } => op_id.as_str(),
        _ => "another variant",
    });
    assert_eq!(op_id, Some("op-3"));
}

#[test]
fn trailers_source_is_the_inner_trailer_error() {
    let error = LandError::Trailers(MortarTrailerError::Git(git_error()));
    let inner = error.source().and_then(|s| s.downcast_ref::<MortarTrailerError>()).map(|e| match e {
        MortarTrailerError::Git(g) => Some(g.clone()),
        MortarTrailerError::ChangeId(_) => None,
    });
    assert_eq!(inner, Some(Some(git_error())));
}

#[test]
fn state_source_is_the_inner_landing_state_error() {
    let error = LandError::State(LandingStateError::Corrupt { path: PathBuf::from("/p"), reason: "why".to_string() });
    let reason = error.source().and_then(|s| s.downcast_ref::<LandingStateError>()).map(|e| match e {
        LandingStateError::Corrupt { reason, .. } => reason.as_str(),
        _ => "another variant",
    });
    assert_eq!(reason, Some("why"));
}

#[test]
fn refusals_land_makes_itself_have_no_source() {
    let refusals = [
        LandError::NoZone { commit: oid("1111111111111111111111111111111111111111") },
        LandError::NoLandingState {
            origin: oid("1111111111111111111111111111111111111111"),
            integration: oid("2222222222222222222222222222222222222222"),
        },
        LandError::IntegrationElsewhere {
            expected: oid("1111111111111111111111111111111111111111"),
            actual: oid("2222222222222222222222222222222222222222"),
        },
        LandError::OpFailed(record()),
        LandError::OpRepeated(record()),
    ];
    for refusal in &refusals {
        assert!(refusal.source().is_none(), "{refusal:?}");
    }
}
