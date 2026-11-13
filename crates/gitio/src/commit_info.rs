//! CommitInfo: what one commit holds. A declaration only; read_commit_info_tests exercises it.

use crate::Oid;

/// A commit's tree, its parents in git's order (none for a root commit) and its full message as git prints it in
/// UTF-8 (re-encoded from a commit that declares another encoding), not trimmed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitInfo {
    pub tree: Oid,
    pub parents: Vec<Oid>,
    pub message: String,
}
