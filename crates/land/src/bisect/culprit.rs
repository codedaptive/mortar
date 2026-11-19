//! The landing bisect names. A bare type; culprit_of_landing_tests exercises it.

use gitio::Oid;
use oplog::ChangeId;

/// The first failing landing on origin.
///
/// `commit` is the landing commit itself, on origin's first-parent line. `change_id` and `zone` are from its
/// squash's Mortar trailers; each is None when that commit carries none, so a commit landed without
/// Mortar is still named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Culprit {
    /// The landing commit on origin's first-parent line.
    pub commit: Oid,
    /// The squash's Mortar-Change-Id trailer.
    pub change_id: Option<ChangeId>,
    /// The squash's Mortar-Zone trailer.
    pub zone: Option<String>,
}
