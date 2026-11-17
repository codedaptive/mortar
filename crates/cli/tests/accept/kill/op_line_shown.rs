//! An op log record as a log line of step 3's block.

use super::OpLine;

impl OpLine {
    /// "op <op_id> <kind> <phase>", then " (<result>)" when it has one, then ": <reason>".
    pub fn shown(&self) -> String {
        let result = if self.result.is_empty() { String::new() } else { format!(" ({})", self.result) };
        format!("op {} {} {}{result}: {}", self.op_id, self.kind, self.phase, self.reason)
    }
}
