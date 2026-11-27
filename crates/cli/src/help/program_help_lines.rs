//! program_help_lines: the program help's text: the usage line, the four groups, the exit codes, then the lines
//! about tokens saved.

use super::{exit_code_help_lines, HELP_GROUPS};
use crate::savings::savings_help_lines;

const PROGRAM_USAGE: &str = "mr <verb> [args] [--json] [--key <operation key>] [--actor <name>]";

/// The usage line; then per group an empty line, its heading, and one row per term: two spaces, the term padded
/// to the longest term of all groups, two spaces, the line. Entries that share a term (install and uninstall,
/// backup and restore-backup) follow each other and print one row. Then exit_code_help_lines: an empty line, the
/// heading, a row per exit code and which code wins. Then savings_help_lines: an empty line and what the tokens
/// saved line is, its switch and its log.
pub fn program_help_lines() -> Vec<String> {
    let width = HELP_GROUPS.iter().flat_map(|group| group.entries).map(|entry| entry.term.len()).max().unwrap_or(0);
    let mut lines = vec![PROGRAM_USAGE.to_string()];
    for group in HELP_GROUPS {
        lines.push(String::new());
        lines.push(group.heading.to_string());
        let mut previous = None;
        for entry in group.entries {
            if previous != Some(entry.term) {
                lines.push(format!("  {:<width$}  {}", entry.term, entry.line));
            }
            previous = Some(entry.term);
        }
    }
    lines.extend(exit_code_help_lines());
    lines.extend(savings_help_lines());
    lines
}
