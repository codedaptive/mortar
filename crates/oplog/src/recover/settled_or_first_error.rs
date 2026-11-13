//! Folds recover's per-intent outcomes into its answer: every closing record, or the first refusal.
//! Pure; settled_or_first_error_tests covers it.

use crate::store::{OpRecord, OplogError};

/// All Ok: the records, in order. Any Err: the first Err in order, and the records are dropped.
/// recover settles every intent before folding, so a refusal never stops the intents after it.
pub(crate) fn settled_or_first_error(
    results: Vec<Result<OpRecord, OplogError>>,
) -> Result<Vec<OpRecord>, OplogError> {
    results.into_iter().collect()
}
