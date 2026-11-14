//! CulpritError as a std error: its source is the inner MortarTrailerError, and NotALanding has none.

use super::CulpritError;

impl std::error::Error for CulpritError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CulpritError::NotALanding { .. } => None,
            CulpritError::Trailers(e) => Some(e),
        }
    }
}
