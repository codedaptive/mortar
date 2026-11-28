//! commit_files: commits named paths of the integration folder onto integration, inside the landing in flight, with
//! no commit hooks run (commit_files_with_hooks is its twin that runs them). It takes no lock: the caller holds
//! `land` and has run recover, as for land and publish.
//! No unit test: its body is commit_files_steps with snapshot's commit_paths, which run git and write the landing
//! state and the op log. Covered by the commit_files_zit tests
//! committing::named_commit_is_published_without_the_unnamed_edit,
//! committing::back_out_drops_the_landing_and_named_commit,
//! committing::append_after_commit_files_uses_the_named_commit_as_parent,
//! committing::same_content_writes_no_op_or_state, refusals::without_a_landing_nothing_is_written,
//! refusals::folder_off_tip_is_refused_without_writes,
//! refusals::absolute_path_passes_through_snapshot_refusal_without_writes,
//! kills::killed_before_update_ref_recovers_failed_then_repeat_moves_once and
//! kills::killed_after_update_ref_recovers_done_repeats_same_and_publishes.

use crate::landing::{commit_files_steps, CommitFilesAnswer, CommitFilesError};
use crate::squash::LandRefs;
use gitio::Git;
use std::path::PathBuf;

/// `git` is rooted at the integration folder (`git.repo`). Reads integration's tip, the landing state and the folder's
/// HEAD commit; decide_commit_files refuses with nothing written when no landing is in flight (NotLanding),
/// integration is not at the commit the state names (NotLanded) or the folder's HEAD is not at the
/// tip (FolderOffTip). Then snapshot's commit_paths makes one commit of the tip's tree with `paths` (relative to the
/// folder) as the folder has them, its message `message`; its refusals are Paths. When the paths as the folder has
/// them equal the tip's tree, the answer is Same with no commit, no ref moved, no op and no landing-state write
/// (snapshot may still store blobs of the folder's files). Committed: the landing state names the commit as pending,
/// one `land-commit-files` op with the newest member's change ID moves integration from the tip to it, and the state
/// then names it as the newest member's integration: Committed { commit }.
/// A run killed before that last write leaves the pending commit, which land, back_out, publish and a repeat of
/// commit_files take as integration only when the op moved integration to it (resolve_landing_state).
pub fn commit_files(
    git: &Git,
    refs: &LandRefs,
    paths: &[PathBuf],
    message: &str,
    actor: &str,
) -> Result<CommitFilesAnswer, CommitFilesError> {
    commit_files_steps(git, refs, actor, |tip| {
        snapshot::commit_paths(git, &git.repo, tip, paths, message).map_err(CommitFilesError::Paths)
    })
}
