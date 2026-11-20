//! How a plan refusal reads: "folder plan: <reason>".

use std::fmt;

use super::PlanError;

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "folder plan: {}", self.reason)
    }
}

impl std::error::Error for PlanError {}
