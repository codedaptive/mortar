//! One op log record as `mr log --json` lists it. Data only: op_lines makes it, and op_lines_tests.rs,
//! op_line_shown_tests.rs, open_intents_tests.rs and done_moves_of_tests.rs cover how it is made and read.

/// The record's op_id, kind, phase ("intent", "done" or "failed"), its result's outcome ("moved", "stale",
/// "not-moved"; "" for an intent, which has none) and reason; each move as (full ref name, new value).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OpLine {
    pub op_id: String,
    pub kind: String,
    pub phase: String,
    pub result: String,
    pub reason: String,
    pub moves: Vec<(String, String)>,
}
