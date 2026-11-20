//! decide_refused_push: every branch with literal ids: remote at new, at expected, elsewhere and absent.

use super::{decide_refused_push, PublishError, PushResult};
use gitio::{Oid, PushRefusal};

const EXPECTED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const OTHER: &str = "3333333333333333333333333333333333333333";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn stale_lease() -> PublishError {
    PublishError::PushRefused { refname: "refs/heads/main".to_string(), refusal: PushRefusal::StaleLease }
}

#[test]
fn remote_at_new_is_pushed() {
    let decided = decide_refused_push(Some(&oid(NEW)), &oid(EXPECTED), &oid(NEW), stale_lease());
    assert!(matches!(decided, Ok(PushResult::Pushed)));
}

#[test]
fn remote_at_expected_returns_the_failure_unchanged() {
    let decided = decide_refused_push(Some(&oid(EXPECTED)), &oid(EXPECTED), &oid(NEW), stale_lease());
    assert!(matches!(
        decided,
        Err(PublishError::PushRefused { ref refname, refusal: PushRefusal::StaleLease }) if refname == "refs/heads/main"
    ));
}

#[test]
fn remote_at_expected_returns_a_remote_missing_failure_unchanged() {
    let failure = PublishError::RemoteMissing { remote: "origin".to_string(), refname: "refs/heads/main".to_string() };
    let decided = decide_refused_push(Some(&oid(EXPECTED)), &oid(EXPECTED), &oid(NEW), failure);
    assert!(matches!(
        decided,
        Err(PublishError::RemoteMissing { ref remote, ref refname })
            if remote == "origin" && refname == "refs/heads/main"
    ));
}

#[test]
fn remote_elsewhere_is_moved() {
    let decided = decide_refused_push(Some(&oid(OTHER)), &oid(EXPECTED), &oid(NEW), stale_lease());
    assert!(matches!(decided, Ok(PushResult::Moved)));
}

#[test]
fn remote_absent_is_moved() {
    let decided = decide_refused_push(None, &oid(EXPECTED), &oid(NEW), stale_lease());
    assert!(matches!(decided, Ok(PushResult::Moved)));
}
