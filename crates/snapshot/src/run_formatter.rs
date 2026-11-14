//! run_formatter: run a snapshot's formatter over the folder's changed files, and say how it ended.
//! No unit test: it runs git and the formatter, and holds no decision of its own (S-5); formatter_paths,
//! formatter_argv, decide_formatter_wait, decide_group_run_end and decide_formatter_result are its decisions and have
//! their tests; run_in_group is its process runner.
//! Nagatha's integration tests cover it through snapshot_formatted, in a throwaway repository:
//! - a formatter that rewrites a changed file (the snapshot tree holds the formatted contents, formatted lists
//!   Formatted with the paths), and is_saved never running the formatter:
//!   tests/snapshot/formatter.rs::rewriting_formatter_runs_once_with_literal_argv_in_folder_and_snapshot_saves_its_output;
//! - one exiting 1 (Failed with code 1 and its stderr, snapshot written):
//!   formatter.rs::exit_one_is_reported_with_code_and_stderr_but_snapshot_still_records_unformatted_files;
//! - a missing program (SpawnFailed, snapshot written):
//!   formatter.rs::missing_formatter_is_spawn_failed_and_does_not_prevent_snapshot_or_log;
//! - a clean folder (runs nothing, formatted None):
//!   formatter.rs::clean_folder_has_no_formatter_invocation_but_first_snapshot_is_written;
//! - no formatter (no git status, formatted None):
//!   formatter.rs::no_formatter_skips_git_status_and_returns_none_while_saving_changed_files;
//! - a deleted file, a refused 6 MiB new file, a file under target/, a caller-excluded file, a conflicted path, a
//!   symlink and a submodule directory among the changes are not passed:
//!   tests/snapshot/formatter_paths.rs::formatter_gets_only_regular_saved_nonconflicted_changes_not_deleted_refused_excluded_links_or_submodules;
//! - the timeout and process-group cases run_in_group's header lists: tests/snapshot/formatter_timeout.rs and
//!   formatter_group.rs.

use crate::decide_formatter_result::decide_formatter_result;
use crate::folder_tree::folder_tree;
use crate::formatter_argv::formatter_argv;
use crate::formatter_paths::formatter_paths;
use crate::run_in_group::run_in_group;
use crate::{Formatter, FormatterRun, SnapshotError};
use gitio::Git;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Runs `formatter` in `folder` over the paths formatter_paths keeps: the changed then the untracked files the
/// snapshot would save differently from HEAD (so not excluded, not refused by `size_cap`), that are regular files now
/// and not conflicted. `git` must run in the folder, as for folder_tree. None when there is no formatter (no git
/// runs) or no path to format. A formatter that fails, cannot start or times out is a listed result, never an Err;
/// only git and folder_tree errors are.
pub(crate) fn run_formatter(
    git: &Git,
    folder: &Path,
    formatter: Option<&Formatter>,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<Option<FormatterRun>, SnapshotError> {
    let Some(formatter) = formatter else {
        return Ok(None);
    };
    let status = git.status_paths(folder).map_err(SnapshotError::Git)?;
    let (head, tree, _) = folder_tree(git, folder, extra_excludes, size_cap)?;
    let saved: HashSet<PathBuf> = git.changed_files(&head, &tree).map_err(SnapshotError::Git)?.into_iter().collect();
    let files: HashSet<PathBuf> = status
        .changed
        .iter()
        .chain(&status.untracked)
        .filter(|p| std::fs::symlink_metadata(folder.join(p)).is_ok_and(|m| m.is_file()))
        .cloned()
        .collect();
    let paths = formatter_paths(&status, &saved, &files);
    let Some(argv) = formatter_argv(Some(formatter), &paths) else {
        return Ok(None);
    };
    Ok(Some(FormatterRun {
        program: formatter.program.clone(),
        args: formatter.args.clone(),
        paths,
        result: decide_formatter_result(run_in_group(&formatter.program, &argv, folder, formatter.timeout)),
    }))
}

