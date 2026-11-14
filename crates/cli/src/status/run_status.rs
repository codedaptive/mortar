//! run_status: `mortar status [--json]`, the agent verb that says where a request folder stands and its next step.
//! No unit test: it runs git through read_status (S-5, S-104); status_report's ZIT crates/cli/tests/
//! status_report_tests.rs and the status ZUTs cover what it reports. Nagatha's integration group is to cover, with
//! the mortar binary as a child process in a request folder made by the landing verbs in a throwaway repository
//! (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - an attempt folder is refused, exit 1;
//! - a zone folder is refused;
//! - any argument is exit 4;
//! - --json gives one object;
//! - status writes no file and moves no ref (refs, index and op log unchanged).

use super::{read_status, status_report};
use crate::args::{usage_report, UsageError};
use crate::config::snapshot_settings_for;
use crate::join::attempt_refusal;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

/// Any argument is usage_report. An attempt folder is refused by attempt_refusal: attempt folders are refused until
/// WO-cli-5 sets each verb's attempt rule. The shell gives an agent verb its folder (actor_for refuses a zone
/// folder) and a verb that needs the config its config; a missing one is a failure here. Then read_status with the
/// config's snapshot settings, reported by status_report. No lock: status is read only (S-85).
pub fn run_status(context: &ShellContext) -> Report {
    let args = &context.args;
    if !args.rest.is_empty() {
        let reason = "status takes no arguments".to_string();
        return usage_report(&UsageError { verb: Some("status".to_string()), reason, json: args.json });
    }
    if let Some(refusal) = context.folder.as_ref().and_then(|folder| attempt_refusal("status", &folder.record)) {
        return refusal;
    }
    let (Some(folder), Some(config)) = (&context.folder, &context.config) else {
        let what = "status runs with its folder and the Mortar config, and the shell gave it neither";
        return failure_report("status", what);
    };
    read_status(folder, &snapshot_settings_for(config), &context.actor, "status")
        .map_or_else(|refusal| refusal, |state| status_report(&state))
}
