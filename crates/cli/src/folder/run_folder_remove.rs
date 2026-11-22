//! run_folder_remove: `mr folder remove <path>`, a joined and saved request folder removed with its branch and its
//! snapshot ref; a saved_state refusal for an attempt folder's unborn HEAD names the try branch (attempt_head_next).
//! No sibling test: it takes a real lock and runs git, the op log, snapshot and folders;
//! folder_removed_report_tests, folder_refusal_tests and folder_error_why_tests cover its decisions. The integration test
//! crates/cli/tests/folder_runs/run_folder_remove_tests.rs covers it through the binary: a joined, saved request folder removed
//! with its branch and snapshot ref by a folder-remove op of the actor; unsaved work, remaining attempts (next naming
//! keep and drop), a zone folder and an attempt folder with an unborn HEAD (next checking out the try branch) each
//! refused exit 1 with nothing removed.
//! Integration tests to write (not written here): a file the snapshot cap refuses counts as unsaved,
//! naming the file and the cap; a request not joined, and a snapshot ref moved under the op, the cli's why and
//! next; a join-<zone> lock held by a live pid, exit 3.

use std::path::{Path, PathBuf};

use folders::make::read_folder_record;
use folders::plan::attempt_branch;
use folders::remove;
use gitio::Git;
use oplog::{KillPidProbe, LockName};
use snapshot::{saved_state, SnapshotError, SnapshotTarget};

use super::{folder_error_report, folder_removed_report};
use crate::config::{snapshot_settings_for, Config};
use crate::join::snapshot_failure;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{attempt_head_next, failure_report, ExitCode, Report};
use crate::save::snapshot_target_for;
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `path` is taken from `cwd` when relative and read as the top of its worktree; git finding no worktree there, or a
/// worktree with no Mortar record, is exit 1 "not-a-folder" before any lock. Under join-<record's zone> after recover
/// (locked_act), keyed by --key or op_key("folder-remove", the folder): a request or attempt folder's saved state is
/// snapshot's saved_state against its own snapshot ref, with the config's excludes and cap, its cap-refused files
/// passed as refused; then folders' remove with the cap, as the actor. A saved_state error is snapshot_failure's
/// report, its next attempt_head_next's when that gives one: in attempt folder k, an unborn HEAD names the try branch
/// (folders' attempt_branch), since the request's branch is not the attempt's. A try branch attempt_branch cannot
/// name leaves snapshot_failure's report as it is.
pub fn run_folder_remove(context: &ShellContext, config: &Config, cwd: &Path, path: &Path) -> Report {
    let path = cwd.join(path);
    let Ok(folder) = Git::new(&path).toplevel() else {
        return not_a_folder(&path, "git finds no worktree there");
    };
    let git_dir = match Git::new(&folder).git_dir(&folder) {
        Ok(git_dir) => git_dir,
        Err(error) => return not_a_folder(&folder, &error.to_string()),
    };
    let record = match read_folder_record(&git_dir) {
        Ok(Some(record)) => record,
        Ok(None) => return not_a_folder(&folder, "the worktree has no Mortar record"),
        Err(error) => return folder_error_report("remove", &error),
    };
    let common = match Git::new(&folder).common_dir() {
        Ok(common) => common,
        Err(error) => return failure_report(VERB, &error.to_string()),
    };
    // Git runs from the common dir, so it still has a repository once the folder is gone, wherever cwd is.
    let git = Git::new(&common);
    let settings = snapshot_settings_for(config);
    let actor = &context.actor;
    let key = op_key(context.args.key.as_deref(), "folder-remove", &folder, &[]);
    let lock = LockName::Join(record.zone.clone());
    locked_act(&git, &common, &lock, &key, &KillPidProbe, VERB, || {
        let (saved, refused): (bool, Vec<PathBuf>) = match snapshot_target_for(&record, actor) {
            Some(target) => {
                let state = saved_state(&git, &folder, &target, &settings.excludes, settings.cap_bytes)
                    .map_err(|error| saved_state_failure(&target, error))?;
                (state.saved, state.refused.into_iter().map(|file| file.path).collect())
            }
            // A zone folder has no snapshot ref; folders' remove refuses it by its kind (ZoneFolder).
            None => (true, Vec::new()),
        };
        remove(&git, &folder, saved, &refused, Some(settings.cap_bytes), actor)
            .map_err(|error| folder_error_report("remove", &error))?;
        Ok(folder_removed_report(&folder, &record))
    })
    .unwrap_or_else(|report| report)
}

/// Exit 1 "not-a-folder": nothing at `path` is a folder Mortar made, for `reason`; nothing was removed.
fn not_a_folder(path: &Path, reason: &str) -> Report {
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "not-a-folder".to_string(),
        lines: vec![format!("{} is not a Mortar folder: {reason}", path.display()), "nothing was removed".to_string()],
        owner: None,
        next: "repeat with the path of a request folder Mortar made".to_string(),
        fields: Vec::new(),
    }
}

/// snapshot_failure for `error`, its next attempt_head_next's when that gives one, with `target`'s try branch when it
/// is an attempt's and attempt_branch can name it. The next is read first: snapshot_failure takes the error.
fn saved_state_failure(target: &SnapshotTarget, error: SnapshotError) -> Report {
    let branch = target.attempt.and_then(|k| attempt_branch(&target.zone, &target.request, k).ok());
    let next = attempt_head_next(&error, branch.as_deref());
    let report = snapshot_failure(VERB, error);
    match next {
        Some(next) => Report { next, ..report },
        None => report,
    }
}
