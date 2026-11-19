//! What bisect answers. A bare type; bisect_report_display_tests exercise its NotFound texts; the Culprit texts
//! are repo integration tests (RITs): tests/land_rit/owed_displays.rs covers them.

use super::{BisectNotFound, Culprit};

/// The first failing landing, or why none can be named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BisectAnswer {
    /// The first failing landing, with the change ID and zone from its squash's trailers when it has them.
    Culprit(Culprit),
    /// No culprit can be named, and why.
    NotFound(BisectNotFound),
}
