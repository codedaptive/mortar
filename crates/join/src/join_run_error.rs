//! JoinRunError: why a join was refused. A declaration only; join_run_error_display_tests exercises each
//! variant's text.
//! There is no Lock variant: join takes no lock; its caller holds join-<zone> (S-47, S-16). oplog's errors are
//! carried as they are, so no Clone or Eq is asked of them.

use crate::JoinError;
use gitio::GitError;
use oplog::{OplogError, OpRecord};
use std::path::PathBuf;

/// A join refusal. Debug only: OplogError and io::Error are neither Clone nor Eq.
#[derive(Debug)]
pub enum JoinRunError {
    /// Replaying the request's commits onto the zone tip failed.
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
    /// The log already held a done op with this attempt's key, so this join moved nothing; that done record.
    OpRepeated(OpRecord),
    /// A conflicted path that is not UTF-8, which mortar-join.json cannot hold without loss.
    StatePath(PathBuf),
    /// Writing mortar-join.json (or its temp file, `path`) failed.
    StateWrite { path: PathBuf, source: std::io::Error },
}
