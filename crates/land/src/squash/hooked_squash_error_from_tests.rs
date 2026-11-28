//! From<SquashError>: the refusal comes back unchanged inside Squash.

use crate::squash::{HookedSquashError, SquashError};

#[test]
fn a_squash_error_becomes_squash_with_the_same_refusal() {
    let error = HookedSquashError::from(SquashError::RefMissing { refname: "refs/heads/x".into() });
    let HookedSquashError::Squash(SquashError::RefMissing { refname }) = error else {
        panic!("expected Squash(RefMissing), got {error:?}");
    };
    assert_eq!(refname, "refs/heads/x");
}
