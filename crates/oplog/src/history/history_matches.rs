//! Whether one op log record meets a history filter.

use super::HistoryFilter;
use crate::store::OpRecord;

/// Ref matches a record with any move of that refname; Key and Actor match the record's field exactly.
pub fn history_matches(record: &OpRecord, filter: &HistoryFilter) -> bool {
    match filter {
        HistoryFilter::Ref(refname) => record.moves.iter().any(|m| m.refname == *refname),
        HistoryFilter::Key(key) => record.key == *key,
        HistoryFilter::Actor(actor) => record.actor == *actor,
    }
}
