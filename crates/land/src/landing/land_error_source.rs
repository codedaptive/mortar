//! LandError as a std error: its source is the wrapped squash, git, op log, trailer or landing state error.

use crate::landing::LandError;

impl std::error::Error for LandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LandError::Squash(e) => Some(e),
            LandError::Git(e) => Some(e),
            LandError::Oplog(e) => Some(e),
            LandError::Trailers(e) => Some(e),
            LandError::State(e) => Some(e),
            LandError::NoZone { .. }
            | LandError::NoLandingState { .. }
            | LandError::IntegrationElsewhere { .. }
            | LandError::OpFailed(_)
            | LandError::OpRepeated(_) => None,
        }
    }
}
