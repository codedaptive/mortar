//! Records one log line. checks_check_tests.rs covers it with the check lines it is ordered among.

use super::{Checks, Line};

impl Checks {
    /// Records a line that is shown but decides nothing.
    pub fn log(&mut self, line: &str) {
        self.lines.push(Line::Log(line.to_string()));
    }
}
