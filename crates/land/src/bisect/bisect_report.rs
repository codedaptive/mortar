//! What bisect reports. A bare type; bisect_report_display_tests exercise its texts: the NotFound answers, the
//! culprit lines and the probe lines.

use super::{BisectAnswer, Probe};

/// The answer, with every probe that led to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BisectReport {
    /// The culprit, or why none can be named.
    pub answer: BisectAnswer,
    /// The probes, in the order they ran.
    pub probes: Vec<Probe>,
}
