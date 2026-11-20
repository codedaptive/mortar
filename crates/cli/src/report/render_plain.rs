//! A report as plain text.

use super::Report;
use crate::savings::{answer_chars, report_savings, savings_line};

/// Each line, then "owner: <owner>" when there is one, then "next: <next>", each ending in "\n"; the last
/// line is always the next step. When the report carries savings, its "[mr] tokens saved" line (against the
/// body's own characters) and an empty line come first. Every other field is --json only.
pub fn render_plain(report: &Report) -> String {
    let mut out = String::new();
    if let Some(savings) = report_savings(report) {
        out.push_str(&savings_line(savings, answer_chars(report)));
        out.push_str("\n\n");
    }
    for line in &report.lines {
        out.push_str(line);
        out.push('\n');
    }
    if let Some(owner) = &report.owner {
        out.push_str(&format!("owner: {owner}\n"));
    }
    out.push_str(&format!("next: {}\n", report.next));
    out
}
