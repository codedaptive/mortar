//! FolderError::source: the wrapped error or io::Error; none for the variants that wrap nothing.

use std::error::Error;
use std::io::ErrorKind;
use std::path::PathBuf;

use gitio::{GitError, Oid};
use oplog::{LockError, OplogError};

use super::FolderError;
use crate::plan::{BelowFloor, PlanError, SweepEstimate};
use crate::record::{FolderKind, FolderRecord, RecordError};

fn source_text(err: &FolderError) -> Option<String> {
    err.source().map(|s| s.to_string())
}

#[test]
fn io_source_is_the_inner_io_error() {
    let err = FolderError::Io {
        path: PathBuf::from("/repo/.git/worktrees/a/mortar-folder.json"),
        source: std::io::Error::new(ErrorKind::PermissionDenied, "denied"),
    };
    let kind = err
        .source()
        .and_then(|s| s.downcast_ref::<std::io::Error>())
        .map(std::io::Error::kind);
    assert_eq!(kind, Some(ErrorKind::PermissionDenied));
}

#[test]
fn git_source_is_the_git_error() {
    let err = FolderError::Git(GitError {
        args: vec!["worktree".to_string()],
        code: Some(128),
        stderr: "fatal: no".to_string(),
    });
    let code = err.source().and_then(|s| s.downcast_ref::<GitError>()).map(|g| g.code);
    assert_eq!(code, Some(Some(128)));
}

#[test]
fn plan_source_is_the_plan_error() {
    let err = FolderError::Plan(PlanError {
        reason: "bad".to_string(),
    });
    assert_eq!(err.source().map(|s| s.to_string()), Some("folder plan: bad".to_string()));
}

#[test]
fn record_source_is_the_record_error() {
    let err = FolderError::Record {
        path: PathBuf::from("/g/mortar-folder.json"),
        error: RecordError {
            reason: "missing key \"kind\"".to_string(),
        },
    };
    let reason = err
        .source()
        .and_then(|s| s.downcast_ref::<RecordError>())
        .map(|r| r.reason.clone());
    assert_eq!(reason, Some("missing key \"kind\"".to_string()));
}

#[test]
fn oplog_source_is_the_oplog_error() {
    let err = FolderError::Oplog(OplogError::Malformed {
        line: None,
        reason: "r".to_string(),
    });
    assert!(err.source().and_then(|s| s.downcast_ref::<OplogError>()).is_some());
    assert_eq!(
        source_text(&err),
        Some("malformed op data: r; mortar did not write this text, so check what did".to_string())
    );
}

#[test]
fn lock_source_is_the_lock_error() {
    let err = FolderError::Lock(LockError::Contended {
        path: PathBuf::from("/r/.git/mortar/locks/x.lock"),
        removed_guards: Vec::new(),
    });
    assert!(err.source().and_then(|s| s.downcast_ref::<LockError>()).is_some());
    assert_eq!(
        source_text(&err),
        Some("/r/.git/mortar/locks/x.lock changed hands repeatedly while acquiring; try again".to_string())
    );
}

#[test]
fn below_floor_source_is_the_below_floor_refusal() {
    let below = BelowFloor {
        free: 5,
        floor: 10,
        sweep: SweepEstimate {
            bytes: 3,
            reason: None,
        },
    };
    let err = FolderError::BelowFloor(below);
    let free = err.source().and_then(|s| s.downcast_ref::<BelowFloor>()).map(|b| b.free);
    assert_eq!(free, Some(5));
}

#[test]
fn variants_wrapping_nothing_have_no_source() {
    let existing = FolderRecord {
        kind: FolderKind::Zone,
        zone: "folders".to_string(),
        request: None,
        base: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        work_order_paths: vec!["crates/folders".to_string()],
        needed_paths: vec![],
        parent: None,
    };
    let none = [
        FolderError::DiskProbe {
            path: PathBuf::from("/wt"),
            reason: "df failed".to_string(),
        },
        FolderError::DifferentFolder {
            path: PathBuf::from("/wt/a"),
            existing,
        },
        FolderError::NotAFolder {
            path: PathBuf::from("/wt/a"),
            reason: "exists and is not a worktree".to_string(),
        },
        FolderError::BranchCheckedOut {
            branch: "zone/folders".to_string(),
            path: PathBuf::from("/wt/b"),
        },
        FolderError::BranchExists {
            branch: "req/a/b".to_string(),
        },
        FolderError::ZoneBranchMissing {
            branch: "zone/folders".to_string(),
        },
        FolderError::NotSaved {
            path: PathBuf::from("/wt/a"),
        },
        FolderError::MoveRefused {
            path: PathBuf::from("/wt/a"),
            paths: vec![PathBuf::from("src/a.rs")],
        },
        FolderError::BranchElsewhere {
            branch: "refs/heads/req/a/b".to_string(),
            at: None,
            old: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
            new: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
        },
    ];
    for err in &none {
        assert!(err.source().is_none(), "{err:?} has a source");
    }
}
