//! Display for SnapshotError: one plain sentence per variant; wrapped errors show through their own Display.

use crate::SnapshotError;
use oplog::{OpRecord, OpResult};
use std::fmt;

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotError::Git(e) => write!(f, "a git run failed: {e}"),
            SnapshotError::Oplog(e) => write!(f, "the op log refused: {e}"),
            SnapshotError::UnbornHead { folder } => {
                write!(f, "HEAD of the folder {} has no commit", folder.display())
            }
            SnapshotError::DetachedHead { folder } => {
                write!(f, "HEAD of the folder {} names no branch", folder.display())
            }
            SnapshotError::BadTarget { reason } => write!(f, "the snapshot ref cannot be named: {reason}"),
            SnapshotError::BadSentence { reason } => write!(f, "the save sentence is refused: {reason}"),
            SnapshotError::ChangeIdMissing { head } => write!(
                f,
                "HEAD {head} is this request's and zone's but carries no Mortar-Change-Id; \
                 no commit was written and no branch moved"
            ),
            SnapshotError::Trailers(e) => write!(f, "HEAD's trailers could not be read: {e}"),
            SnapshotError::ChangeId(e) => write!(f, "a new change ID could not be drawn: {e}"),
            SnapshotError::RefMoved(record) => {
                write!(f, "the snapshot ref {}; nothing moved", stale_text(record))
            }
            SnapshotError::BranchMoved(record) => {
                write!(f, "the branch {}; the branch did not move", stale_text(record))
            }
            SnapshotError::OpRepeated(record) => write!(
                f,
                "the op log already holds a done record for key {} (op {}); nothing moved",
                record.key, record.op_id
            ),
            SnapshotError::IndexNotSet { commit, error } => write!(
                f,
                "the folder's index could not be set to commit {commit}'s tree: {error}; \
                 the files are untouched and the index is the old one"
            ),
            SnapshotError::FileSize { path, error } => write!(
                f,
                "the size of {} could not be read: {error}; no commit was written",
                path.display()
            ),
            // Debug on purpose: a name that is not UTF-8 shows its bytes escaped rather than replaced.
            SnapshotError::PathNotUtf8 { path } => write!(
                f,
                "the refused file {path:?} has a name that is not UTF-8, so it cannot be passed to git as an \
                 exclude; no commit was written"
            ),
            SnapshotError::CapMismatch { paths } => write!(
                f,
                "the capped tree did not differ from the full tree by exactly the refused files: {}; \
                 no commit was written",
                paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
            ),
        }
    }
}

/// The failed record's account of the ref: a Stale result names the ref and what it held, and both are shown; any
/// other result shows only the op id.
fn stale_text(record: &OpRecord) -> String {
    match &record.result {
        Some(OpResult::Stale { refname, actual }) => {
            let held = actual.as_ref().map_or_else(|| "absent".to_string(), |oid| oid.to_string());
            format!("{refname} was not at the value read (it holds {held})")
        }
        _ => format!("was not at the value read (op {})", record.op_id),
    }
}
