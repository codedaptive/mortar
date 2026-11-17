//! Whether a reply ended as a check expects.

use super::Reply;

impl Reply {
    /// True when the run exited `exit` and its report's outcome is `outcome`.
    pub fn is(&self, exit: i32, outcome: &str) -> bool {
        self.exit == Some(exit) && self.outcome == outcome
    }
}
