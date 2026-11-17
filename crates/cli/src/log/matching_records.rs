//! The op records `mr log` lists: those every filter keeps.

use oplog::{history_matches, HistoryFilter, MarkedRecord};

/// The records every filter keeps (AND), in log order; no filters keeps all. Each filter is tested on the record
/// by oplog::history_matches (Ref: any move of that refname, matched whole; Key and Actor: exact).
pub fn matching_records<'a>(records: &'a [MarkedRecord], filters: &[HistoryFilter]) -> Vec<&'a MarkedRecord> {
    records.iter().filter(|marked| filters.iter().all(|filter| history_matches(&marked.record, filter))).collect()
}
