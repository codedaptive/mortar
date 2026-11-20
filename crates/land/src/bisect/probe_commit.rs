//! probe_commit: one probe of the bisect, run in a throwaway detached worktree under the system temp dir.
//! No unit test: it runs git and the caller's command. tests/bisect_zit.rs covers it: every probe's folder is
//! gone and `git worktree list` is clean after a culprit is named
//! (bisect_names_the_merge_landing_that_broke_the_script), after a probe that timed out
//! (a_timed_out_probe_counts_as_a_fail) and after a command that could not be started
//! (a_command_that_cannot_start_stops_the_search).

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use gitio::{Git, Oid};

use super::{probe_outcome_of, BisectError, Probe, ProbeOutcome, TestCommand};

/// The next probe folder's number. Process-wide, so probes run in parallel in one process never share a folder.
static NEXT_FOLDER: AtomicU64 = AtomicU64::new(0);

/// Runs `command` once at `commit` in a new detached worktree, with `timeout` as its limit, and removes the worktree.
///
/// The worktree is removed (forced) whenever it was added, also when filling it or running the command failed; when
/// both the probe and the removal fail, the probe's error is returned. A worktree that could not be added is Git and
/// nothing is removed. A command that could not be run is ProbeCouldNotRun: its verdict is unknown
/// (tests/bisect_zit.rs a_command_that_cannot_start_stops_the_search).
pub(crate) fn probe_commit(
    git: &Git,
    commit: &Oid,
    command: &TestCommand,
    timeout: Duration,
) -> Result<Probe, BisectError> {
    let n = NEXT_FOLDER.fetch_add(1, Ordering::Relaxed);
    let folder = std::env::temp_dir().join(format!("mortar-bisect-{}-{n}", std::process::id()));
    git.worktree_add(&folder, commit, None).map_err(BisectError::Git)?;
    let outcome = run_in_folder(git, commit, command, timeout, &folder);
    let removed = git.worktree_remove(&folder, true).map_err(BisectError::Git);
    let outcome = outcome?;
    removed?;
    Ok(Probe { commit: commit.clone(), outcome, folder })
}

/// Fills `folder` with `commit`'s files and runs `command` there, reading how it ended.
fn run_in_folder(
    git: &Git,
    commit: &Oid,
    command: &TestCommand,
    timeout: Duration,
    folder: &Path,
) -> Result<ProbeOutcome, BisectError> {
    git.worktree_fill(folder).map_err(BisectError::Git)?;
    probe_outcome_of(snapshot::run_in_group(&command.program, &command.args, folder, timeout))
        .map_err(|error| BisectError::ProbeCouldNotRun { commit: commit.clone(), error })
}
