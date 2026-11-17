//! A request verb's setup, before its lock: arguments, the folder's kind, its request and the run's inputs.
//! Shell API: run_join and run_resolve call it.
//! crates/cli/tests/request_run_for_tests.rs covers it (a ZIT: it calls no_arguments_report, attempt_refusal and
//! failure_report for real); it runs no git, since Git::new runs nothing.

use folders::plan::request_branch;
use gitio::Git;
use snapshot::SnapshotTarget;

use super::{attempt_refusal, RequestRun};
use crate::args::no_arguments_report;
use crate::config::snapshot_settings_for;
use crate::lock::lock_name_for;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

/// Any argument is no_arguments_report's usage report for `verb`. Then an attempt folder is attempt_refusal (S-91).
/// A missing folder or config, a record naming no request, a request branch name that cannot be formed, or a verb
/// entry naming no lock is failure_report. Else the run: request_ref "refs/heads/<request_branch>", the verb's lock
/// for the folder's zone, the snapshot target of the request's own folder (attempt None), the config's snapshot
/// settings and the op key of `verb` and the folder with no moves (S-84(4)), unless --key gives one.
pub fn request_run_for<'a>(context: &'a ShellContext, verb: &str) -> Result<RequestRun<'a>, Report> {
    let args = &context.args;
    if let Some(usage) = no_arguments_report(verb, &args.rest, args.json) {
        return Err(usage);
    }
    if let Some(refusal) = context.folder.as_ref().and_then(|folder| attempt_refusal(verb, &folder.record)) {
        return Err(refusal);
    }
    let (Some(folder), Some(config)) = (&context.folder, &context.config) else {
        let what = format!("{verb} runs with its folder and the Mortar config, and the shell gave it neither");
        return Err(failure_report(verb, &what));
    };
    let Some(request) = folder.record.request.as_deref() else {
        return Err(failure_report(verb, "this folder's record names no request"));
    };
    let zone = folder.record.zone.as_str();
    let branch = request_branch(zone, request).map_err(|error| failure_report(verb, &error.to_string()))?;
    let Some(lock) = lock_name_for(args.verb.lock, zone) else {
        return Err(failure_report(verb, &format!("{verb}'s verb entry names no lock")));
    };
    let (zone, request, actor) = (zone.to_string(), request.to_string(), context.actor.clone());
    Ok(RequestRun {
        git: Git::new(&folder.toplevel),
        folder,
        settings: snapshot_settings_for(config),
        target: SnapshotTarget { zone, request, attempt: None, actor },
        request_ref: format!("refs/heads/{branch}"),
        lock,
        key: op_key(args.key.as_deref(), verb, &folder.toplevel, &[]),
    })
}
