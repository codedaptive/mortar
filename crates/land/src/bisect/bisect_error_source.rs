//! BisectError as a std error: its source is the inner GitError or CulpritError, and the refusals have none.

use super::BisectError;

impl std::error::Error for BisectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BisectError::OriginMissing { .. }
            | BisectError::NotOnOrigin { .. }
            | BisectError::OffTheLine { .. }
            | BisectError::ProbeCouldNotRun { .. } => None,
            BisectError::Git(e) => Some(e),
            BisectError::Culprit(e) => Some(e),
        }
    }
}
