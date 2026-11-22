//! The report's plain text: a title, the commit line, one block per Done step, and the result.

use super::{drive_passed, Block, Line, Verdict};

/// The title, then `commit` (commit_line's text, from read_commit) on the second line and a blank line. Each block
/// is its header (`Step <n> <title>: PASS | FAIL | NOT RUN (<reason>)`), one line per recorded line (`ok`, `FAIL`
/// or `log` in a six-wide column after two spaces), then a blank line. A line's text that runs over several lines
/// continues indented by 8 spaces, under the first line's text. Last comes `Result: PASS` or `Result: FAIL`, as
/// drive_passed decides.
pub fn format_report(commit: &str, blocks: &[Block]) -> String {
    let mut text = format!("mortar acceptance report\n{commit}\n\n");
    for block in blocks {
        let verdict = match &block.verdict {
            Verdict::Pass => "PASS".to_string(),
            Verdict::Fail => "FAIL".to_string(),
            Verdict::NotRun(reason) => format!("NOT RUN ({reason})"),
        };
        text.push_str(&format!("Step {} {}: {verdict}\n", block.number, block.title));
        for line in &block.lines {
            let (mark, body) = match line {
                Line::Check { ok: true, did, saw } => ("ok", format!("{did}: {saw}")),
                Line::Check { ok: false, did, saw } => ("FAIL", format!("{did}: {saw}")),
                Line::Log(log) => ("log", log.clone()),
            };
            text.push_str(&format!("  {mark:<6}{}\n", body.replace('\n', "\n        ")));
        }
        text.push('\n');
    }
    text.push_str(if drive_passed(blocks) { "Result: PASS\n" } else { "Result: FAIL\n" });
    text
}
