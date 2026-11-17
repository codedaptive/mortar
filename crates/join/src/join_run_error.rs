//! JoinRunError: why a join, a resolve or a restore of a paused join was refused. A declaration only;
//! join_run_error_display_tests and join_run_error_display_restore_tests exercise each variant's text.
//! There is no Lock variant: join, resolve and restore_join_state take no lock; their caller holds join-<zone>
//! (S-16). oplog's errors are carried as they are, so no Clone or Eq is asked of them.

use crate::{JoinError, MarkedFile};
use gitio::{GitError, Oid};
use oplog::{OplogError, OpRecord};
use std::path::PathBuf;

/// A join, resolve or restore refusal. Debug only: OplogError and io::Error are neither Clone nor Eq.
#[derive(Debug)]
pub enum JoinRunError {
    /// Replaying the request's commits failed: join's replay onto the zone tip, or resolve's continued replay.
    Replay(JoinError),
    /// Reading or writing the op log failed, or a protected zone ref refused a move.
    Oplog(OplogError),
    /// A git run failed.
    Git(GitError),
    /// The zone ref (full name) does not exist; join does not create it.
    ZoneMissing(String),
    /// The request ref (full name) does not exist.
    RequestMissing(String),
    /// The request ref and the zone ref share no history.
    Unrelated { zone_ref: String, request_ref: String },
    /// A ref was not at its expected value, so the op moved nothing; its `failed` record.
    OpFailed(OpRecord),
    /// The log already held a done op with this attempt's key, so this attempt moved nothing; that done record.
    OpRepeated(OpRecord),
    /// A conflicted path that is not UTF-8, which mortar-join.json cannot hold without loss.
    StatePath(PathBuf),
    /// Writing mortar-join.json (or its temp file, `path`) failed.
    StateWrite { path: PathBuf, source: std::io::Error },
    /// resolve found no mortar-join.json at `state`: no join is paused in this folder.
    NothingToResolve { state: PathBuf },
    /// These conflicted files still hold conflict-marker lines; resolve made no commit.
    StillMarked(Vec<MarkedFile>),
    /// The request ref is not at the tip the paused join recorded (`actual` is None when the ref is gone), so the
    /// state at `state` no longer describes the request; it is kept and no ref moved.
    StateStale { request_ref: String, expected: Oid, actual: Option<Oid>, state: PathBuf },
    /// Reading mortar-join.json at `path` failed.
    StateRead { path: PathBuf, source: std::io::Error },
    /// mortar-join.json at `path` is not in the layout join writes; `reason` says where it differs.
    StateCorrupt { path: PathBuf, reason: String },
    /// Removing mortar-join.json at `path` failed after the request ref was moved.
    StateRemove { path: PathBuf, source: std::io::Error },
    /// Reading the conflicted file at `path` to check it for markers failed.
    FileRead { path: PathBuf, source: std::io::Error },
    /// A conflicted path names a folder where resolve needs a file entry, so no resolved commit was made.
    ConflictedPathIsFolder { path: PathBuf },
    /// gitio refused to build the resolved tree from the conflicted paths, so no resolved commit was made; `path` is
    /// the conflicted path the refusal names, when it names one, and `reason` is gitio's.
    ConflictedPathRefused { path: Option<PathBuf>, reason: String },
    /// The op given to restore is not a resolve's final join-rebase op recording a state: another kind, not done,
    /// not exactly one ref move, or no join_state value. Nothing was written; the caller undoes the op as a plain
    /// step, or checks the op id.
    NotAResolveOp { op_id: String },
    /// The state recorded in op `op_id` does not decode, or does not match the op's ref move; `reason` says which.
    /// Nothing was written; the op's ref move can still be undone, and the caller replays the request afresh with a
    /// join.
    RecordedStateCorrupt { op_id: String, reason: String },
    /// The request ref is not at the recorded state's request tip (`actual` is None when the ref is gone), so the
    /// restored state would not describe the request. Nothing was written.
    RestoreStale { request_ref: String, expected: Oid, actual: Option<Oid> },
    /// mortar-join.json at `state` holds a different paused join; it was kept and nothing was written.
    StatePresent { state: PathBuf },
}
