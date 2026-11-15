//! The records of a log that match a history filter.

use super::{history_matches, HistoryFilter};
use crate::store::OpRecord;

/// Every matching record, in log order: Ref matches a record with any move of that refname, Key and
/// Actor match the record's field exactly.
pub fn filter_history<'a>(records: &'a [OpRecord], f: &HistoryFilter) -> Vec<&'a OpRecord> {
    records.iter().filter(|r| history_matches(r, f)).collect()
}
