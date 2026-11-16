//! Whether each attempt of a request is saved, the list folders' keep and drop_attempts take (S-32, S-94).
//! No sibling test (S-104): it reads refs, worktrees, commits and folder trees with git; attempt_saved_tests cover the
//! branch-only rule. The RITs crates/cli/tests/cli_rit/run_folder_keep_tests.rs (saved attempts kept, an unsaved one
//! refused) and run_folder_drop_tests.rs (a branch-only attempt saved by its trysnap tree, one with no trysnap ref refused)
//! cover it through the binary.
//! RITs left for Nagatha (cross-zone, listed only, not written): a file the snapshot cap refuses in an attempt folder
//! counts as unsaved (cli with snapshot's saved_state).

use folders::attempts::attempts_of;
use folders::plan::TRYSNAP_REF_PREFIX;
use gitio::{Git, GitError, Oid};
use snapshot::{saved_state, SnapshotTarget};

use super::attempt_saved;
use crate::config::SnapshotSettings;
use crate::join::snapshot_failure;
use crate::report::{failure_report, Report};

const VERB: &str = "folder";

/// (k, saved) for every attempt of `request` in `zone`, in k order: every branch under git's `refs/heads/` is listed
/// and folders' attempts_of picks the request's try branches and their worktrees, so the cli holds no copy of
/// folders' try-branch naming. An attempt with a folder is saved when snapshot's saved_state on that folder, against
/// its trysnap ref (attempt Some(k)), says saved and no file was refused by the cap. An attempt with a branch and no
/// folder is attempt_saved of its trysnap commit's tree and its tip's tree. A git or snapshot failure is
/// failure_report.
pub fn attempts_saved(
    git: &Git,
    zone: &str,
    request: &str,
    settings: &SnapshotSettings,
    actor: &str,
) -> Result<Vec<(u32, bool)>, Report> {
    let git_failed = |error: GitError| failure_report(VERB, &error.to_string());
    let refs = git.list_refs("refs/heads/").map_err(git_failed)?;
    let worktrees = git.worktree_list().map_err(git_failed)?;
    attempts_of(zone, request, &refs, &worktrees)
        .into_iter()
        .map(|attempt| {
            let saved = match &attempt.folder {
                Some(folder) => {
                    let target = SnapshotTarget {
                        zone: zone.to_string(),
                        request: request.to_string(),
                        attempt: Some(attempt.k),
                        actor: actor.to_string(),
                    };
                    let state = saved_state(git, folder, &target, &settings.excludes, settings.cap_bytes)
                        .map_err(|error| snapshot_failure(VERB, error))?;
                    state.saved && state.refused.is_empty()
                }
                None => {
                    let trysnap = git
                        .read_ref(&format!("{TRYSNAP_REF_PREFIX}{zone}/{request}/{}", attempt.k))
                        .map_err(git_failed)?;
                    let trysnap_tree = trysnap.map(|commit| tree_of(git, &commit)).transpose().map_err(git_failed)?;
                    attempt_saved(trysnap_tree.as_ref(), &tree_of(git, &attempt.tip).map_err(git_failed)?)
                }
            };
            Ok((attempt.k, saved))
        })
        .collect()
}

fn tree_of(git: &Git, commit: &Oid) -> Result<Oid, GitError> {
    git.commit_info(commit).map(|info| info.tree)
}
