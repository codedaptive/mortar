//! run_status: `mortar status [--json]`, the agent verb that says where a request or attempt folder stands and
//! its next step.
//! No unit test: it runs git through read_status (S-5, S-104); status_report's ZIT crates/cli/tests/
//! status_report_tests.rs and the status ZUTs cover what it reports. Nagatha's integration group is to cover, with
//! the mortar binary as a child process in a request folder made by the landing verbs in a throwaway repository
//! (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - in an attempt folder (S-127(1)): exit 0, attempt k's own branch and snapshot, next mortar save or the wait for
//!   the harness, never join (covered by the ZIT crates/cli/tests/status_attempt_folder_tests.rs);
//! - a zone folder is refused;
//! - any argument is exit 4;
//! - --json gives one object;
//! - status writes no file and moves no ref (refs, index and op log unchanged).

use folders::FolderKind;

use super::{attempt_status_report, progress_branch, read_status, status_report, StatusState};
use crate::args::no_arguments_report;
use crate::config::snapshot_settings_for;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

/// Any argument is no_arguments_report's usage report. The shell gives an agent verb its folder (actor_for refuses a
/// zone folder) and a verb that needs the config its config; a missing one is a failure here. Then read_status with
/// the config's snapshot settings, reported by status_report; in attempt folder k it reports attempt k's own branch
/// and snapshot by attempt_status_report, whose next is mortar save or the wait for the harness, never join
/// (S-127(1)). No lock: status is read only (S-85).
pub fn run_status(context: &ShellContext) -> Report {
    let args = &context.args;
    if let Some(usage) = no_arguments_report("status", &args.rest, args.json) {
        return usage;
    }
    let (Some(folder), Some(config)) = (&context.folder, &context.config) else {
        let what = "status runs with its folder and the Mortar config, and the shell gave it neither";
        return failure_report("status", what);
    };
    match read_status(folder, &snapshot_settings_for(config), &context.actor, "status") {
        Err(refusal) => refusal,
        Ok(state) => report_for(folder.record.kind, &state),
    }
}

/// attempt_status_report on attempt k's branch (progress_branch) for an attempt folder, else status_report.
fn report_for(kind: FolderKind, state: &StatusState) -> Report {
    let FolderKind::Attempt(k) = kind else {
        return status_report(state);
    };
    progress_branch(&state.zone, &state.request, kind).map_or_else(
        |error| failure_report("status", &error.reason),
        |branch| attempt_status_report(k, &branch, state),
    )
}
