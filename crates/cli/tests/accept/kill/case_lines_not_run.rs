//! The failed lines for a case's checks that cannot run.

use super::CaseLines;
use crate::report::Checks;

impl CaseLines {
    /// One failed line, saw `not run: <why>`, for each title from the next up to the end of the case's own checks,
    /// or with `consistent` through the consistency items too; the case's block then lists every check.
    pub fn not_run(&mut self, checks: &mut Checks, why: &str, consistent: bool) {
        let end = if consistent { self.titles.len() } else { self.own };
        while self.next < end {
            self.check(checks, false, &format!("not run: {why}"));
        }
    }
}
