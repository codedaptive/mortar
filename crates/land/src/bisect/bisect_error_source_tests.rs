//! source(): the inner GitError for Git, the inner CulpritError for Culprit, none for the refusals.

use std::error::Error;

use gitio::{GitError, Oid};

use super::{BisectError, CulpritError};

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).unwrap()
}

#[test]
fn git_source_is_the_inner_git_error() {
    let err = BisectError::Git(GitError {
        args: vec!["rev-list".to_string()],
        code: Some(128),
        stderr: "fatal: bad revision".to_string(),
    });
    let inner = err.source().and_then(|s| s.downcast_ref::<GitError>());
    assert!(matches!(inner, Some(GitError { code: Some(128), .. })), "{inner:?}");
}

#[test]
fn culprit_source_is_the_inner_culprit_error() {
    let err = BisectError::Culprit(CulpritError::NotALanding { commit: oid('d'), parents: 3 });
    let inner = err.source().and_then(|s| s.downcast_ref::<CulpritError>());
    assert!(matches!(inner, Some(CulpritError::NotALanding { parents: 3, .. })), "{inner:?}");
}

#[test]
fn the_refusals_have_no_source() {
    let refusals = [
        BisectError::OriginMissing { refname: "refs/remotes/origin/main".to_string() },
        BisectError::NotOnOrigin { failed: oid('b'), origin_branch: "origin/main".to_string() },
        BisectError::OffTheLine { last_pass: oid('a'), failed: oid('b') },
        BisectError::ProbeCouldNotRun { commit: oid('c'), error: "spawn failed".to_string() },
    ];
    for err in refusals {
        assert!(err.source().is_none(), "{err:?}");
    }
}
