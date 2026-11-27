//! run_restore: `mr restore <op id> <ref>...`, the harness verb that sets refs back to their values as of an op
//! and moves the folders standing on them.
//! It takes a real lock, runs git, the op log, snapshot and folders; restore_lock_tests,
//! restore_key_tests, restore_refusal_tests, restore_error_report_tests, restore_report_tests and
//! restore_move_refusal_report_tests cover its decisions, run_restore_tests its usage refusals before any lock or
//! git, and the integration test crates/cli/tests/log_restore_runs/run_restore_tests.rs a ref name git refuses (BadMove, exit 3).
//! The integration tests are to cover, with the mortar binary as a child process in a throwaway repository
//! (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - a request folder on req/<z>/<r> is moved to the restored tip;
//! - a zone folder on zone/<z> is listed and not moved;
//! - a restore that would delete a folder's ref refuses exit 3;
//! - mixed refs and an unknown ref are exit 2 before any lock;
//! - a held lock is exit 4.
//!   The integration test crates/cli/tests/log_restore_runs/run_restore_repeat_tests.rs covers an unsaved request folder refusing exit 3
//!   with nothing moved, a kill between oplog::restore and the folder move finished by a plain repeat (Repeated, then
//!   the move), such a repeat with work added after the kill refusing exit 3 with nothing moved, and a
//!   repeat after a later restore leaving the folder and its ref.

use std::path::Path;

use folders::FolderKind;
use gitio::Git;
use oplog::{ApplyOutcome, KillPidProbe, OpCaller};

use super::{
    folders_on_refs, folders_to_move, move_restored_folders, restore_error_report, restore_key, restore_lock,
    restore_moves, restore_refusal, restore_report, FolderOnRef,
};
use crate::args::{usage_report, UsageError};
use crate::config::{snapshot_settings_for, SnapshotSettings};
use crate::context::{repository_error, repository_report};
use crate::lock::locked_act;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "restore";

/// What the act under the lock runs with.
struct Restore<'a> {
    context: &'a ShellContext,
    git: Git,
    common: &'a Path,
    key: String,
    op_id: &'a str,
    refs: &'a [String],
    settings: SnapshotSettings,
}

/// Before any lock or git: the op id and at least one ref (else restore's usage, exit 2), and the one lock the refs
/// choose (restore_lock; a mix or an unknown ref is restore's usage). Then the common dir from `cwd` (no repository:
/// repository_report) and the key (restore_key). Under that lock after recover (locked_act), act runs.
pub fn run_restore(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let usage = |reason: String| usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
    let Some((op_id, refs)) = args.rest.split_first() else {
        return usage("restore needs an op id, then at least one full ref name".to_string());
    };
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before restore runs");
    };
    let lock = match restore_lock(refs, &config.origin_branch, &config.integration_branch) {
        Ok(lock) => lock,
        Err(reason) => return usage(reason),
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(error) => return repository_report(VERB, &repository_error(cwd, error)),
    };
    let key = restore_key(args.key.as_deref(), &common, op_id, refs);
    let run = Restore { context, git, common: &common, key, op_id, refs, settings: snapshot_settings_for(config) };
    locked_act(&run.git, run.common, &lock, &run.key, &KillPidProbe, VERB, || act(&run)).unwrap_or_else(|report| report)
}

/// 1. the folders on the refs (folders_on_refs); 2. the moves, and whether a done record with the key gave them
///    (restore_moves); 3. which request and attempt folders need a move, which a repeat leaves alone because their ref
///    has moved on since its done record, and which are unsaved, checked on a repeat that moved the folder's snapshot
///    ref against that ref's old value from the done record (folders_to_move);
/// 4. restore_refusal, with nothing moved; 5. oplog::restore with the protected refs (a Failed outcome moves no
///    folder); 6. the folders moved from the outcome record's moves (move_restored_folders); 7. restore_report, naming
///    the folders left alone. Zone folders are listed only.
fn act(run: &Restore) -> Result<Report, Report> {
    let (git, common, actor) = (&run.git, run.common, &run.context.actor);
    let oplog_refusal = |error: oplog::OplogError| restore_error_report(&error, &error.to_string());
    let found = folders_on_refs(git, run.refs).map_err(|what| failure_report(VERB, &what))?;
    let (moves, repeated) = restore_moves(git, common, &run.key, run.op_id, run.refs).map_err(oplog_refusal)?;
    let (needing, in_place, unsaved, left_alone) =
        folders_to_move(git, &found, &moves, repeated, &run.settings, actor)?;
    if let Some(refusal) = restore_refusal(&found, &unsaved, &moves) {
        return Err(refusal);
    }
    let caller =
        OpCaller { key: run.key.clone(), actor: actor.clone(), reason: format!("mr restore {}", run.op_id) };
    let outcome = oplog::restore(git, common, run.op_id, run.refs, &caller, &run.context.protected)
        .map_err(oplog_refusal)?;
    let moved = match &outcome {
        ApplyOutcome::Done(record) | ApplyOutcome::Repeated(record) => {
            move_restored_folders(git, &needing, run.op_id, record, actor)?
        }
        ApplyOutcome::Failed(_) => Vec::new(),
    };
    let zone_folders: Vec<FolderOnRef> =
        found.iter().filter(|folder| folder.record.kind == FolderKind::Zone).cloned().collect();
    Ok(restore_report(&outcome, run.op_id, &moved, &in_place, &left_alone, &zone_folders))
}
