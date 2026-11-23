//! commit_files: commits named paths of the integration folder onto integration, inside the landing in flight. It
//! takes no lock: the caller holds `land` and has run recover, as for land and publish.
//! No unit test: it runs git, reads and writes the landing state and writes the op log; decide_commit_files_tests
//! and resolve_landing_state_tests cover its decisions. Covered by the commit_files_zit tests
//! committing::named_commit_is_published_without_the_unnamed_edit,
//! committing::back_out_drops_the_landing_and_named_commit,
//! committing::same_content_writes_no_op_or_state, refusals::without_a_landing_nothing_is_written,
//! refusals::folder_off_tip_is_refused_without_writes,
//! refusals::absolute_path_passes_through_snapshot_refusal_without_writes,
//! kills::killed_before_update_ref_recovers_failed_then_repeat_moves_once and
//! kills::killed_after_update_ref_recovers_done_repeats_same_and_publishes.

use crate::landing::{
    decide_commit_files, integration_op, landing_applied, read_landing_state, write_landing_state, CommitFilesAnswer,
    CommitFilesError, LandError, LandingState,
};
use crate::squash::{read_tip, LandRefs};
use gitio::Git;
use oplog::{apply, new_op_id, unix_now};
use snapshot::CommitPathsOutcome;
use std::path::PathBuf;

/// `git` is rooted at the integration folder (`git.repo`). Reads integration's tip, the landing state and the folder's
/// HEAD commit; decide_commit_files refuses with nothing written when no landing is in flight (NotLanding),
/// integration is not at the commit the state names (NotLanded) or the folder's HEAD is not at the
/// tip (FolderOffTip). Then snapshot's commit_paths writes one commit of the tip's tree with `paths` (relative to the
/// folder) as the folder has them, its message `message`; its refusals are Paths. Same answers Same, with no op and
/// nothing moved. Committed: the landing state names the commit as pending, one `land-commit-files` op moves
/// integration from the tip to it, and the state then names it as the landing's integration: Committed { commit }.
/// A run killed before that last write leaves the pending commit, which land, back_out, publish and a repeat of
/// commit_files take as integration only when the op moved integration to it (resolve_landing_state).
pub fn commit_files(
    git: &Git,
    refs: &LandRefs,
    paths: &[PathBuf],
    message: &str,
    actor: &str,
) -> Result<CommitFilesAnswer, CommitFilesError> {
    let integration_ref = format!("refs/heads/{}", refs.integration_branch);
    let tip = read_tip(git, &integration_ref).map_err(LandError::Squash).map_err(CommitFilesError::Land)?;
    let common = git.common_dir().map_err(LandError::Git).map_err(CommitFilesError::Land)?;
    let state = read_landing_state(&common).map_err(LandError::State).map_err(CommitFilesError::Land)?;
    let (_, folder) = git.head(&git.repo).map_err(LandError::Git).map_err(CommitFilesError::Land)?;
    let resolved = decide_commit_files(state, &tip, folder.as_ref())?;
    let commit = match snapshot::commit_paths(git, &git.repo, &tip, paths, message).map_err(CommitFilesError::Paths)? {
        CommitPathsOutcome::Same => return Ok(CommitFilesAnswer::Same),
        CommitPathsOutcome::Committed { commit, .. } => commit,
    };
    write_landing_state(&common, &LandingState { pending: Some(commit.clone()), ..resolved.clone() })
        .map_err(LandError::State)
        .map_err(CommitFilesError::Land)?;
    let op = integration_op(
        "land-commit-files",
        &integration_ref,
        &tip,
        &commit,
        &resolved.change_id,
        new_op_id(),
        actor,
        unix_now(),
    );
    landing_applied(apply(git, &common, &op)).map_err(CommitFilesError::Land)?;
    write_landing_state(&common, &LandingState { integration: commit.clone(), pending: None, ..resolved })
        .map_err(LandError::State)
        .map_err(CommitFilesError::Land)?;
    Ok(CommitFilesAnswer::Committed { commit })
}
