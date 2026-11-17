//! Records a case's next check line.

use super::CaseLines;
use crate::report::Checks;

impl CaseLines {
    /// One check line titled with the next title, then moves on; returns `ok`. A line past the last title is titled
    /// "a line past the case's checks", so it still shows.
    pub fn check(&mut self, checks: &mut Checks, ok: bool, saw: &str) -> bool {
        let did = self.titles.get(self.next).cloned().unwrap_or_else(|| "a line past the case's checks".to_string());
        self.next += 1;
        checks.check(&did, ok, saw)
    }
}
