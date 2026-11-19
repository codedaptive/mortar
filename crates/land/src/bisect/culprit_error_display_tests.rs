//! Culprit failure text for NotALanding. The Trailers text prints oplog's error: tests/land_rit/displays.rs covers it.

use gitio::Oid;

use super::CulpritError;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }

#[test]
fn not_a_landing_names_the_commit_its_parents_and_what_a_landing_has() {
    assert_eq!(CulpritError::NotALanding { commit: a(), parents: 3 }.to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not a landing: it has 3 parents, and a landing has one (a fast-forward) or two (a merge)");
}
