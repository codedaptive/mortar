//! run_need: `mr need <path> [--key <key>] [--json]`, the agent verb that adds a path to its folder's cone.
//! No unit test: it takes the lock, runs git, the op log and folders::need; need_report_tests,
//! missing_path_report_tests, need_error_report_tests and need_with_next_tests cover what it reports. The validation test
//! crates/cli/tests/validation/run_need_tests.rs covers, with the mortar binary in a throwaway repository:
//! a wrong argument count is usage without work; a missing zone branch refuses before writing
//! (need_with_a_missing_zone_branch_refuses_before_writing); an unreadable record of another folder does not change a
//! completed need (need_does_not_read_other_folders_for_its_next_step). The repo integration tests are to cover, with
//! the mortar binary as a child process in a request folder made by the landing verbs in a throwaway repository
//! (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - need adds a path to the folder's cone and its record, exit 0;
//! - a repeat answers already-done;
//! - a path outside the work order says it is recorded as a gap;
//! - a held join lock is exit 3;
//! - next is the one status gives;
//! - an attempt folder is refused.

use folders::{gaps, need};
use gitio::Git;
use oplog::KillPidProbe;

use super::{need_error_report, need_report, need_with_next, read_need_path};
use crate::args::{usage_report, UsageError};
use crate::config::snapshot_settings_for;
use crate::context::FolderContext;
use crate::join::attempt_refusal;
use crate::lock::{lock_name_for, locked_act};
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;
use crate::status::{read_branch_tips, read_progress};

/// Exactly one argument, else usage_report. An attempt folder is refused by attempt_refusal: keep
/// carries only the attempt's commits, not its needed paths. A missing folder or config is a failure here. Before any
/// lock, so a refusal changes nothing: the path is checked (read_need_path), and so is what the next step reads that
/// need's write cannot change: the record names a request, and the request and zone branches exist
/// (read_branch_tips, missing_branch_report's refusal). Then, under `join-<zone>` from the verb table with recover
/// first (locked_act), folders::need adds it; folders::need takes the folders lock itself, so the cli does
/// not hold it too. The key (--key, else the verb and the folder with no moves) names the lock's holder.
/// Already-done comes from the record read before need; the gap from the record need returns. next is status_next of
/// read_progress for the folder as need left it, which reads no paths and no other folder's record; once need has
/// written, a failed read keeps need's report and gives next "mr status" (need_with_next).
pub fn run_need(context: &ShellContext) -> Report {
    act(context).unwrap_or_else(|report| report)
}

fn act(context: &ShellContext) -> Result<Report, Report> {
    let args = &context.args;
    let [raw] = args.rest.as_slice() else {
        let reason = "need takes one path".to_string();
        return Err(usage_report(&UsageError { verb: Some("need".to_string()), reason, json: args.json }));
    };
    if let Some(refusal) = context.folder.as_ref().and_then(|folder| attempt_refusal("need", &folder.record)) {
        return Err(refusal);
    }
    let (Some(folder), Some(config)) = (&context.folder, &context.config) else {
        let what = "need runs with its folder and the Mortar config, and the shell gave it neither";
        return Err(failure_report("need", what));
    };
    let git = Git::new(&folder.toplevel);
    let clean = read_need_path(&git, &folder.toplevel, raw)?;
    let Some(request) = folder.record.request.as_deref() else {
        return Err(failure_report("need", "this folder's record names no request"));
    };
    read_branch_tips(&git, &folder.record.zone, request, "need")?;
    let Some(lock) = lock_name_for(args.verb.lock, &folder.record.zone) else {
        return Err(failure_report("need", "the verb table gives need no lock; need runs under join-<zone>"));
    };
    let record = locked_act(
        &git,
        &folder.common,
        &lock,
        &op_key(args.key.as_deref(), "need", &folder.toplevel, &[]),
        &KillPidProbe,
        "need",
        || need(&git, &folder.toplevel, &clean, &KillPidProbe).map_err(|error| need_error_report(&error)),
    )?;
    let gap = gaps(&record).contains(&clean);
    let progress = read_progress(
        &FolderContext { record, ..folder.clone() },
        &snapshot_settings_for(config),
        &context.actor,
        "need",
    );
    Ok(need_with_next(
        need_report(&clean, folder.record.needed_paths.contains(&clean), gap, String::new()),
        progress.map(|(state, _)| state),
    ))
}
