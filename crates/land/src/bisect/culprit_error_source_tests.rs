//! source(): the inner MortarTrailerError for Trailers, none for NotALanding.

use std::error::Error;

use gitio::Oid;
use oplog::{ChangeIdError, MortarTrailerError};

use super::CulpritError;

#[test]
fn not_a_landing_has_no_source() {
    let err = CulpritError::NotALanding {
        commit: Oid::parse("89abcdef0123456789abcdef0123456789abcdef").unwrap(),
        parents: 0,
    };
    assert!(err.source().is_none());
}

#[test]
fn trailers_source_is_the_inner_trailer_error() {
    let err = CulpritError::Trailers(MortarTrailerError::ChangeId(ChangeIdError::Malformed {
        text: "abc".to_string(),
        reason: "it has 3 chars".to_string(),
    }));
    let inner = err.source().and_then(|s| s.downcast_ref::<MortarTrailerError>());
    assert!(
        matches!(inner, Some(MortarTrailerError::ChangeId(ChangeIdError::Malformed { text, .. })) if text == "abc"),
        "{inner:?}"
    );
}
