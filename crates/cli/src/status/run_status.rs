//! run_status: `mr status [--json]`, the agent verb that says where a request or attempt folder stands and
//! its next step.
//! No unit test: it runs git through read_status; status_report_tests and the other status unit tests
//! cover what it reports. The integration tests
//! crates/cli/tests/status_need_runs/{status,status_attempt_folder_tests}.rs cover, with the mortar binary as a child
//! process in a request folder made by the landing verbs in a throwaway repository (tests running real git set
//! GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - in an attempt folder: exit 0, attempt k's own branch and snapshot, next mr save or the wait for
//!   the harness, never join (status_attempt_folder_tests.rs);
//! - a zone folder is refused;
//! - any argument is exit 4;
//! - --json gives one object;
//! - status moves no ref and leaves the repository's files alone (refs, index and op log unchanged); its answer
//!   carries savings, so the binary's edge (entry/main_entry, through record_savings) appends one line to the
//!   user's savings.jsonl, outside the repository.
//!
//! crates/cli/tests/status_need_runs/savings.rs covers the "[mr] tokens saved" line first in the plain answer, its
//! baseline equal to git status plus git diff HEAD over the folder, the --json "savings" object, the switch that drops
//! the line, and the savings.jsonl line.

use folders::FolderKind;
use gitio::Git;

use super::{attempt_status_report, progress_branch, read_status, status_queries, status_report, StatusState};
use crate::args::no_arguments_report;
use crate::config::snapshot_settings_for;
use crate::report::{failure_report, Report};
use crate::savings::with_counted_savings;
use crate::shell::ShellContext;

/// Any argument is no_arguments_report's usage report. The shell gives an agent verb its folder (actor_for refuses a
/// zone folder) and a verb that needs the config its config; a missing one is a failure here. Then read_status with
/// the config's snapshot settings, reported by status_report; in attempt folder k it reports attempt k's own branch
/// and snapshot by attempt_status_report, whose next is mr save or the wait for the harness, never join. A Done
/// report gets its savings by with_counted_savings, after its own fields: two more git reads of the whole folder
/// (status_queries: git status and git diff HEAD), their output characters summed by Git::output_chars, whose
/// read-only runs set GIT_OPTIONAL_LOCKS=0 and keep diff.autoRefreshIndex off, so the count leaves the folder's index
/// as it was. A count that fails drops the savings, so the answer goes out without the line; any other report runs
/// no count.
/// No lock: status reads the repository only.
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
        Ok(state) => {
            let git = Git::new(&folder.toplevel);
            with_counted_savings(report_for(folder.record.kind, &state), &status_queries(), &|query| {
                git.output_chars(&folder.toplevel, query)
            })
        }
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
