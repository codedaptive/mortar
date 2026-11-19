//! CommitInfo: what one commit holds. A declaration only; read_commit_info_tests exercises it.

use crate::{Oid, Person};

/// A commit's tree, its parents in git's order (none for a root commit), its author and its full message as git
/// prints it in UTF-8 (re-encoded from a commit that declares another encoding), not trimmed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitInfo {
    pub tree: Oid,
    pub parents: Vec<Oid>,
    /// The author as the commit stores it: name and email unmapped by .mailmap, date as git prints it with
    /// --date=raw (always Some): `<seconds> <offset>` for a well-formed commit, empty when the author line has none,
    /// which GIT_AUTHOR_DATE cannot hand back. For a well-formed commit, passed as commit_tree's author it sets
    /// GIT_AUTHOR_NAME, GIT_AUTHOR_EMAIL and GIT_AUTHOR_DATE to the same values.
    pub author: Person,
    pub message: String,
}
