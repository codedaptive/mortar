//! How a re-review failure reads: git's failure, the trailer failure as it is, or the refusal naming its commits.
//! No sibling unit test (S-104): every test of it runs gitio's Oid Display or another crate's error Display
//! (S-101). No ZIT reaches it yet; tests/land_rit covers the Git and
//! Trailers texts; the NotSameChange and NoParent texts are Nagatha's RITs (owed).

use std::fmt;

use oplog::ChangeId;

use super::RereviewError;

impl fmt::Display for RereviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RereviewError::Git(e) => write!(f, "git failed: {e}"),
            RereviewError::Trailers(e) => write!(f, "{e}"),
            RereviewError::NotSameChange { reviewed, reviewed_id, new, new_id } => write!(
                f,
                "not the same change: {reviewed} carries Mortar-Change-Id {}, {new} carries {}",
                id_text(reviewed_id),
                id_text(new_id)
            ),
            RereviewError::NoParent { commit } => write!(f, "{commit} has no parent to compare against"),
        }
    }
}

/// The id's text, or `none` for a commit with no Mortar-Change-Id.
fn id_text(id: &Option<ChangeId>) -> &str {
    id.as_ref().map_or("none", ChangeId::as_str)
}
