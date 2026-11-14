//! run_formatter: run a snapshot's formatter over the folder's changed files, and say how it ended.
//! No unit test: it runs git and the formatter, and holds no decision of its own (S-5); formatter_paths,
//! formatter_argv, decide_formatter_wait and decide_formatter_result are its decisions and have their tests;
//! run_formatter_child is the formatter's process I/O.
//! Nagatha's integration group should cover, in a throwaway repository: a formatter that rewrites a changed file:
//! the snapshot tree holds the formatted contents and formatted lists Formatted with the paths; a formatter exiting
//! 1: the snapshot is written and formatted lists Failed with code 1 and its stderr; a missing program: SpawnFailed,
//! the snapshot is written; the timeout cases run_formatter_child's header lists (a formatter still running at its
//! timeout is listed TimedOut and the snapshot is written); a deleted file, a refused 6 MB new file, a file under
//! target/, a caller-excluded file, a conflicted path, a symlink and a submodule directory among the changes are not
//! passed; a clean folder runs nothing (formatted None); no formatter runs no git status and gives formatted None;
//! is_saved never runs the formatter.

use crate::decide_formatter_result::decide_formatter_result;
use crate::folder_tree::folder_tree;
use crate::formatter_argv::formatter_argv;
use crate::formatter_paths::formatter_paths;
use crate::run_formatter_child::run_formatter_child;
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
        result: decide_formatter_result(run_formatter_child(&formatter.program, &argv, folder, formatter.timeout)),
    }))
}

