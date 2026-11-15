//! The op records `mortar log` lists: those every filter keeps.

use oplog::{HistoryFilter, OpRecord};

/// The records every filter keeps (AND), in log order; no filters keeps all. The rules are oplog::filter_history's
/// (Ref: any move of that refname, matched whole; Key and Actor: exact), which takes one filter, so the matcher is
/// written here for several.
pub fn matching_records<'a>(records: &'a [OpRecord], filters: &[HistoryFilter]) -> Vec<&'a OpRecord> {
    records.iter().filter(|record| filters.iter().all(|filter| keeps(record, filter))).collect()
}

fn keeps(record: &OpRecord, filter: &HistoryFilter) -> bool {
    match filter {
        HistoryFilter::Ref(refname) => record.moves.iter().any(|m| m.refname == *refname),
        HistoryFilter::Key(key) => record.key == *key,
        HistoryFilter::Actor(actor) => record.actor == *actor,
    }
}
