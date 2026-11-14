//! run_formatter: run a snapshot's formatter over the folder's changed files, and say how it ended.
//! No unit test: it runs git and the formatter, and holds no decision of its own (S-5); formatter_paths,
//! formatter_argv and decide_formatter_result are its decisions and have their tests.
//! Nagatha's integration group should cover, in a throwaway repository: a formatter that rewrites a changed file:
//! the snapshot tree holds the formatted contents and formatted lists Formatted with the paths; a formatter exiting
//! 1: the snapshot is written and formatted lists Failed with code 1 and its stderr; a missing program: SpawnFailed,
//! the snapshot is written; a deleted file, a refused 6 MB new file, a file under target/, a caller-excluded file, a
//! conflicted path, a symlink and a submodule directory among the changes are not passed; a clean folder runs
//! nothing (formatted None); no formatter runs no git status and gives formatted None; is_saved never runs the
//! formatter.

use crate::decide_formatter_result::decide_formatter_result;
use crate::folder_tree::folder_tree;
use crate::formatter_argv::formatter_argv;
use crate::formatter_paths::formatter_paths;
use crate::formatter_stderr_cap::FORMATTER_STDERR_CAP;
use crate::{Formatter, FormatterRun, SnapshotError};
use gitio::Git;
use std::collections::HashSet;
use std::ffi::OsString;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, Command, Stdio};

/// Runs `formatter` in `folder` over the paths formatter_paths keeps: the changed then the untracked files the
/// snapshot would save differently from HEAD (so not excluded, not refused by `size_cap`), that are regular files now
/// and not conflicted. `git` must run in the folder, as for folder_tree. None when there is no formatter (no git
/// runs) or no path to format. A formatter that fails or cannot start is a listed result, never an Err; only git
/// and folder_tree errors are.
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
        result: decide_formatter_result(run_child(&formatter.program, &argv, folder)),
    }))
}

/// Runs `program` with `argv` in `folder`, stdin and stdout null, and gives its exit code (None when killed by a
/// signal) and the first FORMATTER_STDERR_CAP bytes of its stderr. The child is waited for even when reading stderr
/// fails, so it is never left unreaped. An error from spawning, reading stderr or waiting is the Err.
fn run_child(program: &Path, argv: &[OsString], folder: &Path) -> io::Result<(Option<i32>, Vec<u8>)> {
    let mut child = Command::new(program)
        .args(argv)
        .current_dir(folder)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    let stderr = child.stderr.take().map(read_capped).transpose();
    let status = child.wait()?;
    Ok((status.code(), stderr?.unwrap_or_default()))
}

/// The first FORMATTER_STDERR_CAP bytes of `pipe`; the rest is read and dropped, so the child never blocks writing
/// to a full pipe.
fn read_capped(mut pipe: ChildStderr) -> io::Result<Vec<u8>> {
    let mut kept = Vec::new();
    (&mut pipe).take(FORMATTER_STDERR_CAP as u64).read_to_end(&mut kept)?;
    io::copy(&mut pipe, &mut io::sink())?;
    Ok(kept)
}
