//! What bisect reports. A bare type (S-7); bisect_report_display_tests exercise the NotFound texts with no
//! probes; the culprit and probe lines are Nagatha's RITs
//! (owed, S-101).

use super::{BisectAnswer, Probe};

/// The answer, with every probe that led to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BisectReport {
    /// The culprit, or why none can be named.
    pub answer: BisectAnswer,
    /// The probes, in the order they ran.
    pub probes: Vec<Probe>,
}
