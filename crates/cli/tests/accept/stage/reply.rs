//! What one `mr ... --json` run said. Data only: read_reply makes it, and read_reply_tests.rs, reply_text_tests.rs,
//! reply_texts_tests.rs and reply_is_tests.rs cover how it is made and read.

use crate::json::JsonValue;

/// The run's exit code; the report's outcome, next and lines ("" and none when stdout was not a report); the whole
/// report (Null when stdout was not JSON), whose other members text and texts read; and `saw`, the text a check
/// line shows for the run: how it ended, its outcome and, when it did not exit 0, the report's first line.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub exit: Option<i32>,
    pub outcome: String,
    pub next: String,
    pub lines: Vec<String>,
    pub report: JsonValue,
    pub saw: String,
}
