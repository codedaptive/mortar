//! The report of a verb stopped by the git floor check: `Git::version` failed before the verb ran.
//!
//! The check runs for every verb but the exempt ones, so no exempt verb gets this report: help and `--version` never
//! reach run; install and uninstall set up the harness (checks_git_floor says why no git version stops them);
//! guard-hook runs on every ref update, where an extra `git version` process would cost every update.

use gitio::{GitError, Version};

use crate::report::{failure_report, ExitCode, Report};

/// A refusal from the version check itself (no args and no exit code: a git below the floor, or `git version`
/// output it cannot read) is Refused (exit 1), outcome "refused", with gitio's refusal text as the one line (for a
/// git below the floor it names the git found and the floor; for unreadable output it names only the output text);
/// the next step is to install git at gitio's floor (`Version::FLOOR`, as major.minor) or newer. Anything else (git
/// could not be started, or `git version` failed) is a failure report of the error.
pub(crate) fn git_floor_report(verb: &str, error: &GitError) -> Report {
    if !error.args.is_empty() || error.code.is_some() {
        return failure_report(verb, &error.to_string());
    }
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![error.stderr.clone()],
        owner: Some("this machine's git".to_string()),
        next: format!(
            "install git {}.{} or newer, then repeat the command",
            Version::FLOOR.major,
            Version::FLOOR.minor
        ),
        fields: Vec::new(),
    }
}
