//! What happens at the edge to an answer that carries savings.

use std::env::var_os;
use std::fs::read_to_string;
use std::path::Path;

use oplog::unix_now;

use super::{append_savings_log, rfc3339_utc, savings_line_on, savings_log_line, user_dir};
use crate::report::Report;
use crate::savings::{answer_chars, report_savings, tokens_of, without_savings};

/// A report without savings comes back untouched, and nothing is read. One with savings is logged to
/// `<XDG_STATE_HOME or ~/.local/state>/mortar/savings.jsonl` (switch on or off, plain or json) with the time, `cwd`,
/// the verb and the output and baseline tokens; then, for plain output only, the user's
/// `<XDG_CONFIG_HOME or ~/.config>/mortar/config.json` is read and, when its switch is off, the savings are
/// removed so no line is printed. Never fails and never prints.
pub fn record_savings(report: Report, json: bool, cwd: &Path) -> Report {
    let Some(savings) = report_savings(&report) else {
        return report;
    };
    if let Some(state) = user_dir(var_os("XDG_STATE_HOME").as_deref(), var_os("HOME").as_deref(), ".local/state") {
        let output = tokens_of(answer_chars(&report));
        let line = savings_log_line(&rfc3339_utc(unix_now()), cwd, &report.verb, output, tokens_of(savings.baseline_chars));
        append_savings_log(&state, &line);
    }
    if json {
        return report;
    }
    let config = user_dir(var_os("XDG_CONFIG_HOME").as_deref(), var_os("HOME").as_deref(), ".config")
        .and_then(|dir| read_to_string(dir.join("config.json")).ok());
    match savings_line_on(config.as_deref()) {
        true => report,
        false => without_savings(report),
    }
}
