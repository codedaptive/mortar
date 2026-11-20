//! Every way a re-review can fail.
//! A bare type; rereview_error_source_tests exercises each variant, and same_change_tests and
//! plan_rereview_tests the refusals; rereview_error_display_tests the Git, Trailers, NotSameChange and NoParent
//! texts.

use gitio::{GitError, Oid};
use oplog::{ChangeId, MortarTrailerError};

/// A re-review that git could not answer, or that Mortar refuses.
#[derive(Debug)]
pub enum RereviewError {
    /// git could not read a commit or produce the range-diff.
    Git(GitError),
    /// A commit's Mortar trailers could not be read.
    Trailers(MortarTrailerError),
    /// The two commits do not carry one Mortar-Change-Id; None is a commit with no Mortar-Change-Id.
    NotSameChange { reviewed: Oid, reviewed_id: Option<ChangeId>, new: Oid, new_id: Option<ChangeId> },
    /// The commit is a root, so it has no parent to diff from.
    NoParent { commit: Oid },
}
