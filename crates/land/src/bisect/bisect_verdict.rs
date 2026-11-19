//! What the bisect search answers. A bare type; search_first_failing_tests exercises it.

use gitio::Oid;

use super::BisectNotFound;

/// The first failing commit on origin's first-parent line, or why none can be named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BisectVerdict {
    /// A commit that fails the caller's command while its first-parent predecessor passes it.
    FirstFailing(Oid),
    /// No such commit can be named, and why.
    NotFound(BisectNotFound),
}
