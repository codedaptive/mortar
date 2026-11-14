//! LandError: why land, back_out or one of publish's ops was refused; publish wraps it as PublishError::Land. A bare
//! type (S-7); land_error_display_tests exercises the OpFailed and OpRepeated texts,
//! tests/landing_zit (state_display) the State text, land_error_source_tests its source, landing_applied_tests the op
//! variants.

use crate::landing::LandingStateError;
use crate::squash::SquashError;
use gitio::{GitError, Oid};
use oplog::{MortarTrailerError, OpRecord, OplogError};

/// A land, back_out or publish op refusal. Debug only: SquashError, OplogError, MortarTrailerError and
/// LandingStateError are neither Clone nor Eq.
#[derive(Debug)]
pub enum LandError {
    /// Reading the origin or integration tip, or checking the merge's attribute source, failed.
    Squash(SquashError),
    /// A git run failed.
    Git(GitError),
    /// Reading or writing the op log failed, or git failed inside apply.
    Oplog(OplogError),
    /// The squash commit's Mortar trailers could not be read.
    Trailers(MortarTrailerError),
    /// The landing state could not be read, written or removed.
    State(LandingStateError),
    /// The squash commit carries no Mortar-Zone trailer, so its landing message cannot name the zone.
    NoZone { commit: Oid },
    /// Integration is not at origin and no landing state names a landing in flight.
    NoLandingState { origin: Oid, integration: Oid },
    /// back_out (or publish backing out): integration is at neither origin nor the commit the landing state says it
    /// landed.
    IntegrationElsewhere { expected: Oid, actual: Oid },
    /// The ref (integration, the origin or the squash ref) was not at its expected value,
    /// so the op moved nothing; its `failed` record.
    OpFailed(OpRecord),
    /// The log already held a done op with this attempt's key, so this attempt moved nothing; that done record.
    OpRepeated(OpRecord),
}
