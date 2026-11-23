//! WithdrawError: why a withdraw was refused. A bare type; withdraw_error_display_tests exercises the Published,
//! Landing, OpFailed and OpRepeated texts, withdraw_error_source_tests its source, decide_withdraw_tests the
//! Published and Landing decisions.

use crate::landing::LandingStateError;
use crate::squash::SquashError;
use gitio::{GitError, Oid};
use oplog::{ChangeId, OpRecord, OplogError};

/// A withdraw refusal. Debug only: SquashError, OplogError and LandingStateError are neither Clone nor Eq.
#[derive(Debug)]
pub enum WithdrawError {
    /// Reading the origin tip, or the squash ref and its commit's trailers, failed.
    Squash(SquashError),
    /// A git run failed.
    Git(GitError),
    /// Reading or writing the op log failed, or git failed inside apply.
    Oplog(OplogError),
    /// The landing state could not be read.
    State(LandingStateError),
    /// The squash commit is an ancestor of the origin tip `origin`: it was published.
    Published { zone: String, commit: Oid, origin: Oid },
    /// The squash commit is the landing state's squash: it is the landing in flight, with that change ID.
    Landing { zone: String, commit: Oid, change_id: ChangeId },
    /// The squash ref was not at the commit read, so the op moved nothing; its `failed` record.
    OpFailed(OpRecord),
    /// The log already held a done op with this attempt's key, so this withdraw moved nothing; that done record.
    OpRepeated(OpRecord),
}
