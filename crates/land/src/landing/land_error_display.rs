//! Display for LandError (publish's Land variant shows it too): each variant says why and names what it carries;
//! the refusals land and back_out make themselves also say that nothing was moved.

use crate::landing::LandError;
use std::fmt;

impl fmt::Display for LandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LandError::Squash(e) => write!(f, "landing: {e}"),
            LandError::Git(e) => write!(f, "landing git: {e}"),
            LandError::Oplog(e) => write!(f, "landing op log: {e}"),
            LandError::Trailers(e) => write!(f, "landing trailers: {e}"),
            LandError::State(e) => write!(f, "landing: {e}"),
            LandError::NoZone { commit } => write!(
                f,
                "squash commit {commit} carries no Mortar-Zone trailer, so its landing cannot name the zone; \
                 nothing was moved"
            ),
            LandError::NoLandingState { origin, integration } => write!(
                f,
                "integration is at {integration}, not at origin {origin}, and no landing state names a landing in \
                 flight; nothing was moved"
            ),
            LandError::IntegrationElsewhere { expected, actual } => write!(
                f,
                "integration is at {actual}, neither origin nor {expected}, the commit the landing state says was \
                 landed; nothing was moved"
            ),
            LandError::OpFailed(record) => write!(
                f,
                "op {} (key {}) failed: integration was not at its expected value, so nothing was moved; run again \
                 to read the refs afresh",
                record.op_id, record.key
            ),
            LandError::OpRepeated(record) => write!(
                f,
                "op key {} is already done in the op log, so this attempt moved nothing; run again",
                record.key
            ),
        }
    }
}
