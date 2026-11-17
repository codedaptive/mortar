//! The failed lines for the cases an earlier failure left impossible.

use super::{CaseLines, KILL_CASES};
use crate::report::Checks;

/// Every check of each case from `from` (0-based) to the last, consistency items included, as a failed line with
/// saw `not run: <why>`, so the block always lists every case.
pub fn kill_not_run(checks: &mut Checks, from: usize, why: &str) {
    for case in from..KILL_CASES.len() {
        CaseLines::new(case).not_run(checks, why, true);
    }
}
