//! hook_refusal_report: the report of a verb whose commit a pre-commit or commit-msg hook refused, shared by every
//! verb that runs the commit hooks.

use super::{ExitCode, OutValue, Report};

/// Exit Refused, outcome "refused". The lines are the refusal (the hook and its exit code, or a signal when it has
/// none), then `nothing_moved`, then one "stdout: " line per line the hook wrote to stdout and one "stderr: " line
/// per line it wrote to stderr (lossy UTF-8). The owner is whoever maintains the hook; the next is to fix what it
/// reports and run the verb again. The --json fields are hook, code (Null when there is no code or it is negative),
/// and the whole stdout and stderr.
pub(crate) fn hook_refusal_report(verb: &str, refusal: &gitio::HookRefusal, nothing_moved: &str) -> Report {
    let hook = refusal.hook;
    let stdout = String::from_utf8_lossy(&refusal.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&refusal.stderr).into_owned();
    let first = match refusal.code {
        Some(code) => format!("the {hook} hook refused the commit, exit {code}"),
        None => format!("the {hook} hook refused the commit, killed by a signal"),
    };
    let mut lines = vec![first, nothing_moved.to_string()];
    lines.extend(stdout.lines().map(|line| format!("stdout: {line}")));
    lines.extend(stderr.lines().map(|line| format!("stderr: {line}")));
    let code = match refusal.code.and_then(|code| u64::try_from(code).ok()) {
        Some(code) => OutValue::Number(code),
        None => OutValue::Null,
    };
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some(format!("whoever maintains the {hook} hook")),
        next: format!("fix what the {hook} hook reports, then mr {verb} again"),
        fields: vec![
            ("hook".to_string(), OutValue::Text(hook.to_string())),
            ("code".to_string(), code),
            ("stdout".to_string(), OutValue::Text(stdout)),
            ("stderr".to_string(), OutValue::Text(stderr)),
        ],
    }
}
