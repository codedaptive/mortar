//! A log read with values, as the plain op log.

use super::{OpLog, ValuedLog};

/// `log`'s records without their values and its torn_lines, both in the order given.
pub(crate) fn plain_log(log: ValuedLog) -> OpLog {
    OpLog {
        records: log.records.into_iter().map(|v| v.record).collect(),
        torn_lines: log.torn_lines,
    }
}
