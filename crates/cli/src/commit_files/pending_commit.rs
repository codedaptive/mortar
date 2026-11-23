//! PendingCommit: the pending marker of an `mr commit-files` run in a zone folder.
//! A bare type; pending_commit_text_tests and parse_pending_commit_tests exercise it.

use std::path::PathBuf;

use gitio::Oid;

/// The run of mr commit-files in a zone folder that passed the tip check at `tip` for `paths` and has not yet moved
/// zone/<zone>, kept in the folder's git dir as mortar-commit-files-pending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCommit {
    pub tip: Oid,
    pub paths: Vec<PathBuf>,
}
