//! How a below-floor refusal reads: free disk, the floor, and what a sweep would free and why.

use std::fmt;

use super::BelowFloor;

impl fmt::Display for BelowFloor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "free disk {} bytes is under the floor of {} bytes; a sweep would free {} bytes",
            self.free, self.floor, self.sweep.bytes
        )?;
        match &self.sweep.reason {
            Some(reason) => write!(f, " ({reason})"),
            None => Ok(()),
        }
    }
}

impl std::error::Error for BelowFloor {}
