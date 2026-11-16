//! Hands over the recorded lines. checks_new_tests.rs and checks_check_tests.rs read every result through it.

use super::{Checks, Line};

impl Checks {
    pub fn lines(self) -> Vec<Line> {
        self.lines
    }
}
