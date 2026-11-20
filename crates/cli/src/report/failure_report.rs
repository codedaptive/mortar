//! The report of a step that failed (exit 5): a git, disk, lock or op log error a verb has no text of its own for, a
//! verb with no runner (shell/run_in.rs) and a missing config (back_out/run_back_out.rs and others). Install's failed
//! step has its own report (install/apply_failure.rs), as has uninstall's (install/uninstall_failure.rs); install_in
//! builds its unrecorded_failure and state_dir_failure on this one.

use super::{ExitCode, Report};

/// Exit Failure, outcome "failed", no owner: the one line `what`, and next "repeat the same command". It
/// claims nothing about what the failed step left behind; a verb that knows adds its own lines.
pub fn failure_report(verb: &str, what: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![what.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}
