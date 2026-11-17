//! A logged call as a check line lists it.

use super::LogCall;

impl LogCall {
    /// The argv joined by spaces, then " (parent <parent>)".
    pub fn shown(&self) -> String {
        format!("{} (parent {})", self.args.join(" "), self.parent)
    }
}
