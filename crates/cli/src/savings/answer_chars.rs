//! The characters of an answer's own output.

use super::without_savings;
use crate::report::Report;
use crate::render_plain;

/// The character count (chars, not bytes) of the report's plain form with its savings fields removed: the
/// answer's own body, the one output size the plain line, --json and the savings log all use.
pub fn answer_chars(report: &Report) -> u64 {
    u64::try_from(render_plain(&without_savings(report.clone())).chars().count()).unwrap_or(u64::MAX)
}
