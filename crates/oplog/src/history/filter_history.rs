//! The records of a log that match a history filter.

use super::HistoryFilter;
use crate::store::OpRecord;

/// Every matching record, in log order: Ref matches a record with any move of that refname, Key and
/// Actor match the record's field exactly.
pub fn filter_history<'a>(records: &'a [OpRecord], f: &HistoryFilter) -> Vec<&'a OpRecord> {
    records.iter().filter(|r| matches(r, f)).collect()
}

fn matches(r: &OpRecord, f: &HistoryFilter) -> bool {
    match f {
        HistoryFilter::Ref(refname) => r.moves.iter().any(|m| m.refname == *refname),
        HistoryFilter::Key(key) => r.key == *key,
        HistoryFilter::Actor(actor) => r.actor == *actor,
    }
}
