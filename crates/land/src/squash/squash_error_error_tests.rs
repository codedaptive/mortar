//! source(): the wrapped error for Git, Oplog, Trailers and ChangeId; none for the refusals squash makes itself.

use super::SquashError;
use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OplogError};
use std::error::Error;

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
        kind: "land-squash".to_string(),
        actor: "a".to_string(),
        reason: "r".to_string(),
        at: 1,
        phase: OpPhase::Failed,
        moves: Vec::new(),
        result: None,
    }
}

#[test]
fn git_source_is_the_inner_git_error() {
    let error = SquashError::Git(git_error());
    assert_eq!(error.source().and_then(|s| s.downcast_ref::<GitError>()), Some(&git_error()));
}

#[test]
fn oplog_source_is_the_inner_oplog_error() {
    let error = SquashError::Oplog(OplogError::UnknownOp { op_id: "op-3".to_string() });
    let op_id = error.source().and_then(|s| s.downcast_ref::<OplogError>()).map(|e| match e {
        OplogError::UnknownOp { op_id } => op_id.as_str(),
        _ => "another variant",
    });
    assert_eq!(op_id, Some("op-3"));
}

#[test]
fn trailers_source_is_the_inner_trailer_error() {
    let error = SquashError::Trailers(MortarTrailerError::Git(git_error()));
    let inner = error.source().and_then(|s| s.downcast_ref::<MortarTrailerError>()).map(|e| match e {
        MortarTrailerError::Git(g) => Some(g.clone()),
        MortarTrailerError::ChangeId(_) => None,
    });
    assert_eq!(inner, Some(Some(git_error())));
}

#[test]
fn change_id_source_is_the_inner_change_id_error() {
    let error = SquashError::ChangeId(ChangeIdError::Malformed { text: "abc".to_string(), reason: "r".to_string() });
    let text = error.source().and_then(|s| s.downcast_ref::<ChangeIdError>()).map(|e| match e {
        ChangeIdError::Malformed { text, .. } => text.as_str(),
        ChangeIdError::Random { .. } => "Random",
    });
    assert_eq!(text, Some("abc"));
}

#[test]
fn refusals_squash_makes_itself_have_no_source() {
    let refusals = [
        SquashError::RefMissing { refname: "refs/heads/main".to_string() },
        SquashError::AttrSourceMissing { commit: oid("1111111111111111111111111111111111111111") },
        SquashError::Unrelated {
            origin: oid("1111111111111111111111111111111111111111"),
            source: oid("2222222222222222222222222222222222222222"),
        },
        SquashError::OpFailed(record()),
        SquashError::OpRepeated(record()),
    ];
    for refusal in &refusals {
        assert!(refusal.source().is_none(), "{refusal:?}");
    }
}
