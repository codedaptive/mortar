//! The report of an `mr guard` that land's guard_install or guard_status stopped, in the cli's own words: land's
//! and gitio's Display texts are never printed.
//! The Git case is an integration test to write (not written here): it needs git itself to fail to name
//! the hooks folder or common dir, and checks the cli's words are printed, not gitio's.

use land::GuardError;

use super::foreign_hook_report;
use crate::report::{failure_report, ExitCode, Report};

const VERB: &str = "guard";
const IO_REPEAT: &str = "what was already written is the guard's own and harmless; a repeat writes only what is \
                         missing or different";

/// - ForeignHook { path }: exit 3 Refused, outcome "foreign-hook", lines "<path> is a hook Mortar did not write;
///   Mortar never overwrites one" and "nothing was written", owner "the repository's own hooks", next moving `path`
///   aside, then `mr guard install --actor <name>` (foreign_hook_report). Fields: none.
/// - BadConfig { path, why }: exit 2 Usage (a configuration problem), outcome "bad-config", line "<path> is not a
///   Mortar guard config: <why>", owner "the Mortar guard", next `mr guard install --actor <name>` (install
///   rewrites a guard.json whose bytes differ from the config's). Fields: none.
/// - Io { path, error }: failure_report (exit 5, "failed", next "repeat the same command"), lines "reading or writing
///   <path> failed: <error>" and then that what was written is harmless and a repeat writes only what is missing
///   or different (install can fail after writing guard.json).
/// - Git: failure_report, "git could not name the repository's hooks folder or common dir".
pub fn guard_error_report(error: &GuardError) -> Report {
    match error {
        GuardError::ForeignHook { path } => {
            let mut report = foreign_hook_report(std::slice::from_ref(path));
            report.lines.push("nothing was written".to_string());
            report
        }
        GuardError::BadConfig { path, why } => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Usage,
            outcome: "bad-config".to_string(),
            lines: vec![format!("{} is not a Mortar guard config: {why}", path.display())],
            owner: Some("the Mortar guard".to_string()),
            next: "mr guard install --actor <name>".to_string(),
            fields: Vec::new(),
        },
        GuardError::Io { path, error } => {
            // guard_install writes guard.json before the hooks, so a failed hook write can leave guard.json behind.
            let mut report = failure_report(VERB, &format!("reading or writing {} failed: {error}", path.display()));
            report.lines.push(IO_REPEAT.to_string());
            report
        }
        GuardError::Git(_) => failure_report(VERB, "git could not name the repository's hooks folder or common dir"),
    }
}
