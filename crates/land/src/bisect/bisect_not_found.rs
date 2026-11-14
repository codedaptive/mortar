//! Why bisect can name no culprit (07-bisect.md). A bare type (S-7); search_first_failing_tests exercises it.

/// The reason no first failing commit can be named between the last passing commit and the failed one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BisectNotFound {
    /// The failed commit passes the caller's command, so nothing in the range fails it.
    FailedPasses,
    /// The last passing commit fails the caller's command, so the break is not in the range.
    LastPassFails,
}
