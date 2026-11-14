//! How a re-review failure reads: git's failure, the trailer failure as it is, or the refusal naming its commits.

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
            RereviewError::BasesDiffer { reviewed_parent, new_parent } => write!(
                f,
                "different parents: reviewed on {reviewed_parent}, new on {new_parent}; re-review across a moved \
                 origin is not supported yet"
            ),
        }
    }
}

/// The id's text, or `none` for a commit with no Mortar-Change-Id.
fn id_text(id: &Option<ChangeId>) -> &str {
    id.as_ref().map_or("none", ChangeId::as_str)
}
