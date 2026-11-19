//! run_folder_keep: `mr folder keep <k>`, run in a request folder: attempt k kept, the request branch and folder
//! moved to it, every attempt folder and try branch gone, their trysnap refs kept.
//! No sibling test: it takes a real lock and runs git, the op log, snapshot and folders;
//! request_folder_refusal_tests, pending_keep_move_tests, attempt_saved_tests and kept_report_tests cover its
//! decisions. The RIT crates/cli/tests/cli_rit/run_folder_keep_tests.rs covers it through the binary: keep 2 of two saved
//! attempts by a folder-keep op of the actor; an unsaved attempt, and unsaved work in the request folder, refused
//! exit 1 with nothing changed; an orphan attempt re-made and the keep of another finished in one command; a keep
//! killed between its op and the request folder's move finished by a plain repeat, outcome already-done.
//! Repo integration tests to write (cross-zone, not written here): the request branch moved on after the attempts were
//! cut, and a try branch moved under the op, the cli's why and next; an orphan with no trysnap ref, exit 1 naming its
//! folder; a file the snapshot cap refuses in an attempt counts as unsaved; a join-<zone> lock held by a live pid,
//! exit 3.

use std::path::Path;

use folders::plan::{attempt_branch, request_branch};
use folders::{keep, move_folder, remake_orphans, FolderError};
use gitio::{Git, GitError};
use oplog::{KillPidProbe, LockName};
use snapshot::{saved_state, snapshot, SnapshotTarget};

use super::{attempts_saved, folder_error_report, kept_report, pending_keep_move, request_folder_refusal};
use crate::config::{snapshot_settings_for, Config};
use crate::context::{context_report, read_folder_context, FolderContext};
use crate::join::snapshot_failure;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, ExitCode, Report};
use crate::save::snapshot_target_for;
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `cwd` must be in a request folder (read_folder_context; a zone or attempt folder is request_folder_refusal's exit
/// 1). Under join-<zone> after recover (locked_act), keyed by --key or op_key("folder-keep", the folder), as the actor:
/// 1. with try branch k gone and the op log holding keep k's op with the request branch still at its new tip
///    (pending_keep_move), the folder's move a killed keep left is finished: the folder snapshotted, then
///    moved from the op's old tip to its new one; outcome already-done. A file the snapshot's cap refused does not
///    stop the move: move_folder keeps untracked files, so it stays in the folder untouched, as join's
///    finish_pending_move does;
/// 2. the request folder's own work must be saved, with no file the cap refused, else exit 1 and nothing runs;
/// 3. folders' remake_orphans, before 4; 4. attempts_saved; 5. folders' keep; 6. the request
///    folder moved from the request branch's old tip to the kept one, saved as step 2 found it.
pub fn run_folder_keep(context: &ShellContext, config: &Config, cwd: &Path, k: u32) -> Report {
    let folder = match read_folder_context(cwd) {
        Ok(folder) => folder,
        Err(error) => return context_report(VERB, &error),
    };
    if let Some(refusal) = request_folder_refusal("folder keep <k>", &folder.record) {
        return refusal;
    }
    let Some(target) = snapshot_target_for(&folder.record, &context.actor) else {
        return failure_report(VERB, "the request folder's record names no request");
    };
    let git = Git::new(&folder.toplevel);
    let key = op_key(context.args.key.as_deref(), "folder-keep", &folder.toplevel, &[]);
    let lock = LockName::Join(target.zone.clone());
    locked_act(&git, &folder.common, &lock, &key, &KillPidProbe, VERB, || {
        keep_under_lock(&git, &folder, &target, config, k)
    })
    .unwrap_or_else(|report| report)
}

fn keep_under_lock(
    git: &Git,
    folder: &FolderContext,
    target: &SnapshotTarget,
    config: &Config,
    k: u32,
) -> Result<Report, Report> {
    let (zone, request, actor) = (&target.zone, &target.request, &target.actor);
    let refused = |error: FolderError| folder_error_report("keep", &error);
    let git_failed = |error: GitError| failure_report(VERB, &error.to_string());
    let settings = snapshot_settings_for(config);
    let (excludes, cap) = (&settings.excludes, settings.cap_bytes);
    let plan_refused = |error| refused(FolderError::Plan(error));
    let request_ref = format!("refs/heads/{}", request_branch(zone, request).map_err(plan_refused)?);
    let kept_branch = format!("refs/heads/{}", attempt_branch(zone, request, k).map_err(plan_refused)?);
    if git.read_ref(&kept_branch).map_err(git_failed)?.is_none() {
        let records =
            oplog::read_op_log(&folder.common).map_err(|error| failure_report(VERB, &error.to_string()))?.records;
        let branch_at = git.read_ref(&request_ref).map_err(git_failed)?;
        if let Some((old, new)) = pending_keep_move(&records, &request_ref, &kept_branch, branch_at.as_ref()) {
            // The snapshot's refused list is not a refusal here: move_folder keeps untracked files, so a cap-refused
            // file stays in the folder untouched.
            snapshot(git, &folder.toplevel, target, excludes, cap).map_err(|error| snapshot_failure(VERB, error))?;
            move_folder(git, &folder.toplevel, &old, &new, true, actor, None).map_err(refused)?;
            return Ok(kept_report(k, &old, &new, &[], true));
        }
    }
    let state =
        saved_state(git, &folder.toplevel, target, excludes, cap).map_err(|error| snapshot_failure(VERB, error))?;
    if !state.saved || !state.refused.is_empty() {
        return Err(request_unsaved(&folder.toplevel));
    }
    // remake_orphans first: an orphan has no branch for saved_state to read until it is re-made.
    // It sets each orphan's index, re-applies its cone (which may write that attempt's files) and makes its try
    // branch; it drops nothing and a repeat finds them made. keep then refuses an unsaved attempt before it removes
    // any folder or moves any branch.
    let remade = remake_orphans(git, zone, request, actor).map_err(refused)?;
    let saved = attempts_saved(git, zone, request, &settings, actor)?;
    let (old, new) = keep(git, zone, request, k, &saved, actor).map_err(refused)?;
    move_folder(git, &folder.toplevel, &old, &new, true, actor, None).map_err(refused)?;
    Ok(kept_report(k, &old, &new, &remade, false))
}

/// Exit 1 "refused": the request folder at `toplevel` holds work that is not saved (or a file the cap refused); nothing
/// ran.
fn request_unsaved(toplevel: &Path) -> Report {
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("the request folder {} holds work that is not saved", toplevel.display()),
            "nothing ran".to_string(),
        ],
        owner: None,
        next: "repeat once the request folder's work is saved".to_string(),
        fields: Vec::new(),
    }
}
