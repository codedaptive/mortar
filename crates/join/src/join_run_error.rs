//! JoinRunError: why a join or a resolve was refused. A declaration only; join_run_error_display_tests exercises
//! each variant's text.
//! There is no Lock variant: join and resolve take no lock; their caller holds join-<zone> (S-47, S-16). oplog's
//! errors are carried as they are, so no Clone or Eq is asked of them.

use crate::{JoinError, MarkedFile};
use gitio::{GitError, Oid};
use oplog::{OplogError, OpRecord};
use std::path::PathBuf;

/// A join or resolve refusal. Debug only: OplogError and io::Error are neither Clone nor Eq.
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
}
