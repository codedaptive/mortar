//! The usage report of a verb that takes no arguments and was given some.

use super::{usage_report, UsageError};
use crate::report::Report;

/// None when `rest` is empty; else usage_report for `verb` with the reason "<verb> takes no arguments", in the
/// form `json` asks for.
pub fn no_arguments_report(verb: &str, rest: &[String], json: bool) -> Option<Report> {
    if rest.is_empty() {
        return None;
    }
    Some(usage_report(&UsageError {
        verb: Some(verb.to_string()),
        reason: format!("{verb} takes no arguments"),
        json,
    }))
}
