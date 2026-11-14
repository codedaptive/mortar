//! Every way naming a landing's culprit can fail.
//! A bare type (S-7); culprit_error_source_tests and culprit_of_landing_tests exercise each variant;
//! Display texts: tests/land_rit covers Trailers; NotALanding is Nagatha's RIT (owed).

use gitio::Oid;
use oplog::MortarTrailerError;

/// A failure reading the culprit from a landing commit.
#[derive(Debug)]
pub enum CulpritError {
    /// The commit has neither one parent (a fast-forward) nor two (a merge, S-58).
    NotALanding { commit: Oid, parents: usize },
    /// The squash's Mortar trailers could not be read.
    Trailers(MortarTrailerError),
}
