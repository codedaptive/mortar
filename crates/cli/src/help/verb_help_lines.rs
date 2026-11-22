//! verb_help_lines: one verb's help text.

use super::{usage_forms, verb_notes, HelpEntry};
use crate::savings::verb_baseline;

/// The verb's usage, one line per form, then its line from the program help, then its notes (verb_notes); for a
/// verb with a baseline (verb_baseline), then "tokens saved is counted against: <baseline>".
pub fn verb_help_lines(entry: &HelpEntry) -> Vec<String> {
    let mut lines = usage_forms(entry.name);
    lines.push(entry.line.to_string());
    lines.extend(verb_notes(entry.name).iter().map(|note| note.to_string()));
    lines.extend(verb_baseline(entry.name).map(|words| format!("tokens saved is counted against: {words}")));
    lines
}
