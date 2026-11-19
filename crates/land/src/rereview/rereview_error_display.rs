//! How a re-review failure reads: git's failure, the trailer failure as it is, or the refusal naming its commits.
//! rereview_error_display_tests cover the NotSameChange and NoParent texts; tests/land_rit/displays.rs covers the
//! Git and Trailers texts. No ZIT reaches it yet.

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
