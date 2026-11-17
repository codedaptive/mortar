//! save_run_for: what `mr save` and `mr undo` read once, before their lock, as join's RequestRun, so that
//! each can finish a pending folder move with join's finish_pending_move (S-143).
//! Shell API: run_save and run_undo call it.

use folders::plan::{attempt_branch, request_branch};
use folders::FolderKind;
use gitio::Git;

use super::snapshot_target_for;
use crate::config::snapshot_settings_for;
use crate::join::RequestRun;
use crate::lock::lock_name_for;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

/// The run of `verb` (save or undo) in a request folder or attempt folder k; neither is refused (an attempt saves to
/// and undoes on its own branch, S-69, S-142). target is snapshot_target_for's (attempt k's for attempt folder k);
/// request_ref is "refs/heads/" and the folder's own branch: the request branch for a request folder, attempt k's
/// try branch for attempt folder k; the lock is the verb entry's for the folder's zone (lock_name_for); the key is
/// --key, else `verb`'s op key of the folder with no moves (S-84), which names the lock's holder only (S-90). A
/// missing folder or config, a record naming no request, a branch name that cannot be formed or a verb entry naming
/// no lock is failure_report for `verb`.
pub fn save_run_for<'a>(context: &'a ShellContext, verb: &str) -> Result<RequestRun<'a>, Report> {
    let Some(folder) = &context.folder else {
        return Err(failure_report(verb, &format!("{verb} runs in its folder, and the shell gave it none")));
    };
    let Some(config) = &context.config else {
        return Err(failure_report(verb, &format!("{verb} runs with the Mortar config, and the shell gave it none")));
    };
    let Some(target) = snapshot_target_for(&folder.record, &context.actor) else {
        return Err(failure_report(verb, "this folder's record names no request; actor_for refuses a zone folder"));
    };
    let branch = match folder.record.kind {
        FolderKind::Attempt(k) => attempt_branch(&target.zone, &target.request, k),
        FolderKind::Request | FolderKind::Zone => request_branch(&target.zone, &target.request),
    }
    .map_err(|error| failure_report(verb, &format!("the folder's branch name cannot be formed: {}", error.reason)))?;
    let Some(lock) = lock_name_for(context.args.verb.lock, &target.zone) else {
        let what = format!("the verb table gives {verb} no lock; {verb} moves a ref and needs join-<zone>");
        return Err(failure_report(verb, &what));
    };
    Ok(RequestRun {
        git: Git::new(&folder.toplevel),
        folder,
        settings: snapshot_settings_for(config),
        target,
        request_ref: format!("refs/heads/{branch}"),
        lock,
        key: op_key(context.args.key.as_deref(), verb, &folder.toplevel, &[]),
    })
}
