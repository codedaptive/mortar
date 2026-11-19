//! verb_help_lines: one verb's help text.

use super::{usage_forms, HelpEntry};

/// The verb's usage, one line per form, then its line from the program help.
pub fn verb_help_lines(entry: &HelpEntry) -> Vec<String> {
    let mut lines = usage_forms(entry.name);
    lines.push(entry.line.to_string());
    lines
}
