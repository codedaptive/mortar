//! SquashError: why a squash or would_merge was refused. A declaration only (S-7); squash_error_display_tests
//! exercise the texts of RefMissing, OpFailed and OpRepeated, tests/land_rit those of Git, Oplog, Trailers and
//! ChangeId; the AttrSourceMissing and Unrelated texts are Nagatha's RITs (owed, S-101).
//! squash_error_error_tests exercise its source, squash_applied_tests the op variants.

use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpRecord, OplogError};

/// A squash or would_merge refusal. Debug only: OplogError and ChangeIdError are neither Clone nor Eq.
#[derive(Debug)]
pub enum SquashError {
    /// A git run failed.
    Git(GitError),
    /// Reading or writing the op log failed, the squash ref's name was refused, or git failed inside apply.
    Oplog(OplogError),
    /// The Mortar trailers of the prior squash commit could not be read.
    Trailers(MortarTrailerError),
    /// A new change ID could not be drawn, or a zone commit's Mortar-Change-Id is malformed.
    ChangeId(ChangeIdError),
    /// The origin, zone or integration ref (full name) does not exist.
    RefMissing { refname: String },
    /// The commit named as the merge's attribute source does not exist.
    AttrSourceMissing { commit: Oid },
    /// The zone tip and the origin tip share no history.
    Unrelated { origin: Oid, source: Oid },
    /// The squash ref was not at its expected value, so the op moved nothing; its `failed` record.
    OpFailed(OpRecord),
    /// The log already held a done op with this attempt's key, so this squash moved nothing; that done record.
    OpRepeated(OpRecord),
}
