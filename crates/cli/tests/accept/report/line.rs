//! One line of a step's block in the report. Data only: format_report_tests.rs and block_verdict_tests.rs cover
//! how it is shown and judged.

/// A check (what was done, whether it held, what was seen) or a log line that decides nothing, such as an op log
/// line step 3 copies into its block.
#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Check { ok: bool, did: String, saw: String },
    Log(String),
}
