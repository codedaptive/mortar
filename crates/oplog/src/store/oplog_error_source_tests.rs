//! source(): the inner io::Error for Io, the GitError for Git, nothing for the rest.

use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;

use gitio::GitError;

use super::OplogError;

#[test]
fn io_source_is_the_inner_io_error() {
    let err = OplogError::Io {
        path: PathBuf::from("/repo/.git/mortar/oplog.jsonl"),
        source: std::io::Error::new(ErrorKind::PermissionDenied, "denied here"),
    };
    let kind = err
        .source()
        .and_then(|s| s.downcast_ref::<std::io::Error>())
        .map(std::io::Error::kind);
    assert_eq!(kind, Some(ErrorKind::PermissionDenied));
}

#[test]
fn git_source_is_the_git_error() {
    let err = OplogError::Git(GitError {
        args: vec!["update-ref".to_string()],
        code: Some(1),
        stderr: "boom".to_string(),
    });
    let inner = err.source().and_then(|s| s.downcast_ref::<GitError>());
    assert_eq!(
        inner,
        Some(&GitError {
            args: vec!["update-ref".to_string()],
            code: Some(1),
            stderr: "boom".to_string(),
        })
    );
}

#[test]
fn malformed_and_mixed_have_no_source() {
    let malformed = OplogError::Malformed {
        line: Some(1),
        reason: "x".to_string(),
    };
    let mixed = OplogError::Mixed {
        op_id: "o".to_string(),
        key: "k".to_string(),
        refs: Vec::new(),
    };
    assert!(malformed.source().is_none());
    assert!(mixed.source().is_none());
}

#[test]
fn bad_move_has_no_source() {
    let err = OplogError::BadMove {
        refname: "refs/heads/a".to_string(),
        reason: "is listed twice".to_string(),
    };
    assert!(err.source().is_none());
}

#[test]
fn protect_undo_and_restore_refusals_have_no_source() {
    let zero = gitio::Oid::parse("0000000000000000000000000000000000000000").unwrap();
    let errs = [
        OplogError::Protected {
            refname: "refs/heads/main".to_string(),
            current: None,
            new: zero,
            by: "land".to_string(),
        },
        OplogError::NothingToUndo {
            refs: vec!["refs/heads/a".to_string()],
        },
        OplogError::MovedSince {
            op_id: "o".to_string(),
            refname: "refs/heads/a".to_string(),
            left: None,
            actual: None,
            moved_by: None,
        },
        OplogError::UnknownOp {
            op_id: "o".to_string(),
        },
        OplogError::ValueUnknown {
            op_id: "o".to_string(),
            refname: "refs/heads/a".to_string(),
        },
    ];
    for err in &errs {
        assert!(err.source().is_none(), "{err:?}");
    }
}
