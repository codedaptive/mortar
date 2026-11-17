//! run_folder_drop: `mr folder drop`, run in a request folder: every attempt of the request dropped, their folders
//! and try branches gone, the request branch unmoved and the trysnap refs kept (S-82, S-94).
//! No sibling test (S-104): it takes a real lock and runs git, the op log, snapshot and folders;
//! request_folder_refusal_tests, attempt_saved_tests and dropped_report_tests cover its decisions. The RIT
//! crates/cli/tests/cli_rit/run_folder_drop_tests.rs covers it through the binary: saved attempts dropped by an op of the actor
//! with the request unmoved and their snapshots kept; a branch-only attempt whose trysnap tree equals its tip's tree
//! dropped; one with no trysnap ref refused exit 1 naming its branch, nothing changed.
//! RITs left for Nagatha (cross-zone, listed only, not written): a try branch moved under the op, the cli's why and
//! next; an orphan attempt folder with no trysnap ref, exit 1 naming the folder; a join-<zone> lock held by a live pid,
//! exit 3.

use std::path::Path;

use folders::{drop_attempts, remake_orphans, FolderError};
use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{attempts_saved, dropped_report, folder_error_report, request_folder_refusal};
use crate::config::{snapshot_settings_for, Config};
use crate::context::{context_report, read_folder_context};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `cwd` must be in a request folder (read_folder_context; a zone or attempt folder is request_folder_refusal's exit
/// 1). Under join-<zone> after recover (locked_act), keyed by --key or op_key("folder-drop", the folder), as the actor:
/// folders' remake_orphans, then attempts_saved, then folders' drop_attempts, in that order (S-96 (2), S-131).
pub fn run_folder_drop(context: &ShellContext, config: &Config, cwd: &Path) -> Report {
    let folder = match read_folder_context(cwd) {
        Ok(folder) => folder,
        Err(error) => return context_report(VERB, &error),
    };
    if let Some(refusal) = request_folder_refusal("folder drop", &folder.record) {
        return refusal;
    }
    let Some(request) = folder.record.request.as_deref() else {
        return failure_report(VERB, "the request folder's record names no request");
    };
    let (zone, actor) = (&folder.record.zone, &context.actor);
    let git = Git::new(&folder.toplevel);
    let settings = snapshot_settings_for(config);
    let key = op_key(context.args.key.as_deref(), "folder-drop", &folder.toplevel, &[]);
    let lock = LockName::Join(zone.clone());
    locked_act(&git, &folder.common, &lock, &key, &KillPidProbe, VERB, || {
        let refused = |error: FolderError| folder_error_report("drop", &error);
        // remake_orphans first (S-96 (2), S-131): an orphan has no branch for saved_state to read until it is re-made.
        // It sets each orphan's index, re-applies its cone (which may write that attempt's files) and makes its try
        // branch; it drops nothing and a repeat finds them made. drop_attempts then refuses an unsaved attempt before
        // it removes any folder or branch.
        let remade = remake_orphans(&git, zone, request, actor).map_err(refused)?;
        let saved = attempts_saved(&git, zone, request, &settings, actor)?;
        let dropped = drop_attempts(&git, zone, request, &saved, actor).map_err(refused)?;
        Ok(dropped_report(&dropped, &remade))
    })
    .unwrap_or_else(|report| report)
}
