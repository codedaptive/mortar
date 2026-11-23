//! RecordedBase: what a join op recorded about the request's base. A declaration only; recorded_base_tests exercises
//! it through recorded_base. Not named recorded_base.rs: the function recorded_base holds that name.

use crate::request_base::RequestBase;

/// The request base a join-rebase or join-zone-request op recorded under BASE_BEFORE and BASE_AFTER.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordedBase {
    /// The op recorded both values: `before`, the base the request was replayed from (the folder record's base and
    /// partners before the join), and `after`, the base it now stands on.
    Recorded { before: RequestBase, after: RequestBase },
    /// The op is not a join-rebase or join-zone-request op, or lacks either value, as ops written before join
    /// recorded the base do. Not an error: there is no base to read.
    NotRecorded,
    /// The op carries both values but one does not decode; `reason` names the key and what is wrong with its text.
    Unreadable { reason: String },
}
