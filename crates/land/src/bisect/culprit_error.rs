//! Every way naming a landing's culprit can fail.
//! A bare type; culprit_error_source_tests and culprit_of_landing_tests exercise each variant;
//! Display texts: culprit_error_display_tests cover NotALanding, tests/land_rit Trailers.

use gitio::Oid;
use oplog::MortarTrailerError;

/// A failure reading the culprit from a landing commit.
#[derive(Debug)]
pub enum CulpritError {
    /// The commit has neither one parent (a fast-forward) nor two (a merge).
    NotALanding { commit: Oid, parents: usize },
    /// The squash's Mortar trailers could not be read.
    Trailers(MortarTrailerError),
}
