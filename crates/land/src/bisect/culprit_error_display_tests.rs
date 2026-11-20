//! Culprit failure texts for NotALanding and Trailers; the Trailers text prints oplog's error.

use gitio::Oid;
use oplog::{ChangeIdError, MortarTrailerError};

use super::CulpritError;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn bad_id() -> ChangeIdError {
    ChangeIdError::Malformed { text: "bad".into(), reason: "expected 32 characters".into() }
}

#[test]
fn not_a_landing_names_the_commit_its_parents_and_what_a_landing_has() {
    assert_eq!(CulpritError::NotALanding { commit: a(), parents: 3 }.to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not a landing: it has 3 parents, and a landing has one (a fast-forward) or two (a merge)");
}

#[test]
fn culprit_trailers_display_is_the_inner_oplog_change_id_text_without_a_land_prefix() {
    assert_eq!(CulpritError::Trailers(MortarTrailerError::ChangeId(bad_id())).to_string(),
        "not a change ID: \"bad\": expected 32 characters");
}
