//! A marked log as a valued log: each record with its values, its remote mark dropped.

use crate::store::{ValuedLog, ValuedRecord};

use super::MarkedLog;

/// Every record with its values, in order, without its mark; torn_lines unchanged.
pub(crate) fn unmarked_log(log: MarkedLog) -> ValuedLog {
    ValuedLog {
        records: log
            .records
            .into_iter()
            .map(|m| ValuedRecord {
                record: m.record,
                values: m.values,
            })
            .collect(),
        torn_lines: log.torn_lines,
    }
}
