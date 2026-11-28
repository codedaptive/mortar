//! commit_files_steps: the steps commit_files and commit_files_with_hooks share, with the commit of the named paths
//! made by the caller's `commit`. It takes no lock: the caller holds `land` and has run recover.
//! No unit test: it runs git, reads and writes the landing state and writes the op log; decide_commit_files_tests
//! and resolve_landing_state_tests cover its decisions. In a throwaway repository, the commit_files_zit tests listed
//! in commit_files.rs cover it without hooks, and tests/commit_files_zit/hooks.rs with them.

use crate::landing::{
    decide_commit_files, integration_op, landing_applied, read_landing_state, write_landing_state, CommitFilesAnswer,
    CommitFilesError, LandError, LandingMember, LandingState,
};
use crate::squash::{read_tip, LandRefs};
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};
use snapshot::CommitPathsOutcome;

/// `git` is rooted at the integration folder (`git.repo`). Reads integration's tip, the landing state and the folder's
/// HEAD commit; decide_commit_files refuses with nothing written when no landing is in flight (NotLanding),
/// integration is not at the commit the state names (NotLanded) or the folder's HEAD is not at the tip
/// (FolderOffTip). Then `commit` gets the tip and makes the commit of the named paths onto it in the folder, or a
/// refusal. Same answers Same with no commit, no ref moved, no op and no landing-state write. Committed: the landing
/// state names the commit as pending, one `land-commit-files` op with the newest member's change ID moves
/// integration from the tip to it, and the state then names it as the newest member's integration:
/// Committed { commit }. Every CommitFilesError becomes E.
pub(crate) fn commit_files_steps<E: From<CommitFilesError>>(
    git: &Git,
    refs: &LandRefs,
    actor: &str,
    commit: impl FnOnce(&Oid) -> Result<CommitPathsOutcome, E>,
) -> Result<CommitFilesAnswer, E> {
    let integration_ref = format!("refs/heads/{}", refs.integration_branch);
    let tip = read_tip(git, &integration_ref).map_err(LandError::Squash).map_err(CommitFilesError::Land)?;
    let common = git.common_dir().map_err(LandError::Git).map_err(CommitFilesError::Land)?;
    let state = read_landing_state(&common).map_err(LandError::State).map_err(CommitFilesError::Land)?;
    let (_, folder) = git.head(&git.repo).map_err(LandError::Git).map_err(CommitFilesError::Land)?;
    let resolved = decide_commit_files(state, &tip, folder.as_ref())?;
    let commit = match commit(&tip)? {
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
        &resolved.newest.change_id,
        new_op_id(),
        actor,
        unix_now(),
    );
    landing_applied(apply(git, &common, &op)).map_err(CommitFilesError::Land)?;
    let newest = LandingMember { integration: commit.clone(), ..resolved.newest };
    write_landing_state(&common, &LandingState { newest, pending: None, ..resolved })
        .map_err(LandError::State)
        .map_err(CommitFilesError::Land)?;
    Ok(CommitFilesAnswer::Committed { commit })
}
