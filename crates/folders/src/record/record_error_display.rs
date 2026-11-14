//! How a folder record refusal reads, and RecordError as a std error with no source.

use std::fmt;

use super::RecordError;

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "folder record: {}", self.reason)
    }
}

impl std::error::Error for RecordError {}
