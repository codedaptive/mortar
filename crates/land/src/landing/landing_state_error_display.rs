//! Display for LandingStateError: each variant says what failed and names the path; Corrupt also says why.

use crate::landing::LandingStateError;
use std::fmt;

impl fmt::Display for LandingStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LandingStateError::Read { path, source } => {
                write!(f, "landing state {} could not be read: {source}", path.display())
            }
            LandingStateError::Corrupt { path, reason } => {
                write!(f, "landing state {} is not a state land wrote: {reason}", path.display())
            }
            LandingStateError::Write { path, source } => {
                write!(f, "landing state {} could not be written: {source}", path.display())
            }
            LandingStateError::Remove { path, source } => {
                write!(f, "landing state {} could not be removed: {source}", path.display())
            }
        }
    }
}
