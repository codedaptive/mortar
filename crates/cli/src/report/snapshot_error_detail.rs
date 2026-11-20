//! snapshot_error_detail: cli's one line for a SnapshotError, snapshot_step_failure's line about the failed step: the
//! first line of its report, or the second after the line for what earlier steps moved.
//! It sits beside snapshot_step_failure, its one caller.

use oplog::{OpRecord, OpResult};
use snapshot::SnapshotError;

/// What the failed step left, scoped to that step: a no-movement or no-commit claim speaks for the failed snapshot or
/// op or attempt only, since an earlier step of the same verb (the pending move's snapshot, the formatter's, save's
/// own) may have moved the snapshot ref, and the pending move may have moved the folder's files. A wrapped git, op
/// log, trailers or change ID error is its own Display, without SnapshotError's prefix. ChangeIdMissing names HEAD and
/// claims nothing about movement: the caller's next line says what that step left. BranchMoved names the branch and
/// what it held, and likewise claims nothing. IndexNotSet says the index step left the files untouched. UnbornHead,
/// DetachedHead, BadTarget and BadSentence are snapshot's Display, which claims nothing about movement.
pub(crate) fn snapshot_error_detail(error: &SnapshotError) -> String {
    match error {
        SnapshotError::Git(inner) => inner.to_string(),
        SnapshotError::Oplog(inner) => inner.to_string(),
        SnapshotError::Trailers(inner) => inner.to_string(),
        SnapshotError::ChangeId(inner) => inner.to_string(),
        SnapshotError::ChangeIdMissing { head } => {
            format!("HEAD {head} is this request's and zone's but carries no Mortar-Change-Id")
        }
        SnapshotError::RefMoved(record) => {
            format!("the snapshot ref {}; this snapshot moved nothing", stale_text(record))
        }
        SnapshotError::OpRepeated(record) => format!(
            "the op log already holds a done record for key {} (op {}); this attempt moved nothing",
            record.key, record.op_id
        ),
        SnapshotError::FileSize { path, error } => format!(
            "the size of {} could not be read: {error}; {NO_SNAPSHOT_COMMIT}",
            path.display()
        ),
        // Debug on purpose, as snapshot's Display: a name that is not UTF-8 shows its bytes escaped.
        SnapshotError::PathNotUtf8 { path } => format!(
            "the refused file {path:?} has a name that is not UTF-8, so it cannot be passed to git as an exclude; \
             {NO_SNAPSHOT_COMMIT}"
        ),
        SnapshotError::CapMismatch { paths } => format!(
            "the capped tree did not differ from the full tree by exactly the refused files: {}; {NO_SNAPSHOT_COMMIT}",
            paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
        ),
        SnapshotError::BranchMoved(record) => format!("the branch {}", stale_text(record)),
        SnapshotError::IndexNotSet { commit, error } => format!(
            "the folder's index could not be set to commit {commit}'s tree: {error}; the index step left the files \
             untouched, and the index is the old one, or that tree without its skip-worktree bits if the \
             sparse-checkout reapply failed"
        ),
        SnapshotError::UnbornHead { .. }
        | SnapshotError::DetachedHead { .. }
        | SnapshotError::BadTarget { .. }
        | SnapshotError::BadSentence { .. } => error.to_string(),
    }
}

/// FileSize, PathNotUtf8 and CapMismatch stop the snapshot step before it writes its commit.
const NO_SNAPSHOT_COMMIT: &str = "this snapshot wrote no commit";

/// The failed record's account of the ref: a Stale result names the ref and what it held (absent when nothing);
/// any other result names only the op.
fn stale_text(record: &OpRecord) -> String {
    match &record.result {
        Some(OpResult::Stale { refname, actual }) => {
            let held = actual.as_ref().map_or_else(|| "absent".to_string(), ToString::to_string);
            format!("{refname} was not at the value read (it holds {held})")
        }
        _ => format!("was not at the value read (op {})", record.op_id),
    }
}
