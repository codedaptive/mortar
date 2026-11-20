//! One attempt folder make_attempts is to make.
//! A bare type; plan_attempts_tests exercises it.

use crate::record::FolderRecord;

/// Attempt `k` (from 1), on `branch` (the short name, `try/<zone>/<request>/<k>`), with the record its folder gets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedAttempt {
    pub k: u32,
    pub branch: String,
    pub record: FolderRecord,
}
