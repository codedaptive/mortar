//! A report as plain text.

use super::Report;

/// Each line, then "owner: <owner>" when there is one, then "next: <next>", each ending in "\n"; the last
/// line is always the next step. Fields are --json only.
pub fn render_plain(report: &Report) -> String {
    let mut out = String::new();
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
