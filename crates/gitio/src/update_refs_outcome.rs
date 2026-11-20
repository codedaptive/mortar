//! UpdateRefsOutcome: what an all-or-nothing ref update did. A declaration only; classify_update_refs_tests
//! exercises it.

/// Every ref moved, or one ref was not at its expected value and nothing moved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateRefsOutcome {
    /// Every ref moved.
    Updated,
    /// This ref was not at its expected value, and nothing moved.
    Stale { refname: String },
}
