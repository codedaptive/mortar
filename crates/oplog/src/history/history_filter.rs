//! Which records a history query keeps.
//! A bare type; filter_history_tests and history_matches_tests exercise each variant.

/// One criterion on an op log record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryFilter {
    /// Records with any move of this refname.
    Ref(String),
    /// Records with this key.
    Key(String),
    /// Records with this actor.
    Actor(String),
}
