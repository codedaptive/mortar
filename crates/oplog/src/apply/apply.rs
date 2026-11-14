//! Applies one op with no values: apply_with_values with none.
//! No unit test: it reads and appends the log file and runs git (S-5) through apply_with_values, whose header
//! names the tests covering its decisions and lists what Nagatha's integration group should cover.

use std::path::Path;

use gitio::Git;

use super::{apply_with_values, ApplyOutcome};
use crate::store::{Op, OplogError};

/// apply_with_values(git, common, op, &[]), whose doc is the full contract: the intent and the closing record
/// are written with no values. A Repeated still answers with the done record; its values are dropped.
pub fn apply(git: &Git, common: &Path, op: &Op) -> Result<ApplyOutcome, OplogError> {
    apply_with_values(git, common, op, &[]).map(|v| v.outcome)
}
