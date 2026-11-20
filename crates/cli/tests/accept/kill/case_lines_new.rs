//! A case's titles, ready for its first line.

use super::{CaseLines, CONSISTENT_ITEMS, KILL_CASES};

impl CaseLines {
    /// The titles of KILL_CASES[case] (0-based), then CONSISTENT_ITEMS, each as "<label> <title>".
    pub fn new(case: usize) -> CaseLines {
        let (label, own) = KILL_CASES[case];
        let titles = own.iter().chain(CONSISTENT_ITEMS.iter()).map(|title| format!("{label} {title}")).collect();
        CaseLines { titles, own: own.len(), next: 0 }
    }
}
