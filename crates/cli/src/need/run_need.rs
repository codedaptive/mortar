//! run_need: `mortar need <path> [--key <key>] [--json]`, the agent verb that adds a path to its folder's cone.
//! No unit test: it takes the lock, runs git, the op log and folders::need (S-5, S-104); need_report_tests,
//! missing_path_report_tests and need_error_report_tests cover what it reports. Nagatha's integration group is to
//! cover, with the mortar binary as a child process in a request folder made by the landing verbs in a throwaway
//! repository (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - need adds a path to the folder's cone and its record, exit 0;
//! - a repeat answers already-done;
//! - a path outside the work order says it is recorded as a gap;
//! - a held join lock is exit 3;
//! - next is the one status gives;
//! - an attempt folder is refused.

use folders::{gaps, need};
use gitio::Git;
use oplog::KillPidProbe;

use super::{need_error_report, need_report, read_need_path};
use crate::args::{usage_report, UsageError};
use crate::config::snapshot_settings_for;
use crate::context::FolderContext;
use crate::join::attempt_refusal;
use crate::lock::{lock_name_for, locked_act};
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;
use crate::status::{read_status, status_next};

/// Exactly one argument, else usage_report. An attempt folder is refused by attempt_refusal: attempt folders are
/// refused until WO-cli-5 sets each verb's attempt rule. A missing folder or config is a failure here. The path is
/// checked before any lock (read_need_path): a refusal there changes nothing. Then, under `join-<zone>` from the
/// verb table with recover first (locked_act, S-84(3)), folders::need adds it; folders::need takes the folders lock
/// itself, so the cli does not hold it too. The key (--key, else the verb and the folder with no moves) names the
/// lock's holder. Already-done comes from the record read before need; the gap from the record need returns; next is
/// the one status gives for the folder as need left it.
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
    Ok(need_report(
        &clean,
        folder.record.needed_paths.contains(&clean),
        gaps(&record).contains(&clean),
        status_next(&read_status(
            &FolderContext { record, ..folder.clone() },
            &snapshot_settings_for(config),
            &context.actor,
            "need",
        )?),
    ))
}
