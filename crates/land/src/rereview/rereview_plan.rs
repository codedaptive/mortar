//! What a re-review shows. A bare type (S-7); plan_rereview_tests exercises it.

use gitio::Oid;

/// The re-review plan_rereview decides on.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RereviewPlan {
    /// Same tree and same parents: there is no diff to show.
    NothingChanged,
    /// git's range-diff of `reviewed_base..reviewed` against `new_base..new`; the two bases are each commit's own
    /// first parent, equal when the two share one.
    RangeDiff { reviewed_base: Oid, new_base: Oid },
}
