//! Failure texts: NotALanding names the commit, its parent count and what a landing has.
//! The Trailers text is oplog's own failure text, so it is left to Nagatha's cross-zone tests.

use gitio::Oid;

use super::CulpritError;

#[test]
fn not_a_landing_names_the_commit_its_parents_and_what_a_landing_has() {
    let err = CulpritError::NotALanding {
        commit: Oid::parse("89abcdef0123456789abcdef0123456789abcdef").unwrap(),
        parents: 3,
    };
    assert_eq!(
        err.to_string(),
        "89abcdef0123456789abcdef0123456789abcdef is not a landing: it has 3 parents, and a landing has one \
         (a fast-forward) or two (a merge, S-58)"
    );
}
