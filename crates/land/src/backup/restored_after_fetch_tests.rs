//! restored_after_fetch: Created at the fetched id; Vanished when the remote had no ref or no copy was read.

use super::{restored_after_fetch, Restored};
use gitio::{FetchRefOutcome, Oid};

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn fetched_with_an_id_is_created_at_that_id() {
    let fetched = Some(oid("3333333333333333333333333333333333333333"));
    assert_eq!(
        restored_after_fetch("refs/heads/zone/a".to_string(), FetchRefOutcome::Fetched, fetched),
        Restored::Created {
            refname: "refs/heads/zone/a".to_string(),
            oid: oid("3333333333333333333333333333333333333333"),
        }
    );
}

#[test]
fn missing_source_is_vanished() {
    assert_eq!(
        restored_after_fetch("refs/mortar/snap/s1".to_string(), FetchRefOutcome::MissingSource, None),
        Restored::Vanished { refname: "refs/mortar/snap/s1".to_string() }
    );
}

#[test]
fn fetched_with_no_copy_read_is_vanished() {
    assert_eq!(
        restored_after_fetch("refs/heads/req/land/r1".to_string(), FetchRefOutcome::Fetched, None),
        Restored::Vanished { refname: "refs/heads/req/land/r1".to_string() }
    );
}
