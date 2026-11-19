//! resolved_commit: the commit a resolve makes from the resolved tree. resolved_commit_tests covers the commit
//! written and the refusal of an empty author date.

use crate::join_state::JoinState;
use crate::replay_git::ReplayGit;
use crate::JoinError;
use gitio::Oid;

/// Writes a commit of `resolved_tree` (built by continue_replay) on the state's replayed tip, with the conflicted
/// commit's message (trailers and change ID kept) and author (its raw date unchanged); the committer is left to
/// git's own identity, as replay_commit does. A conflicted commit whose author date is empty is refused before any
/// commit is written, since commit_tree cannot hand an empty date back unchanged.
pub(crate) fn resolved_commit(
    git: &impl ReplayGit,
    state: &JoinState,
    resolved_tree: &Oid,
) -> Result<Oid, JoinError> {
    let info = git.commit_info(&state.commit)?;
    if info.author.date.as_deref() == Some("") {
        return Err(JoinError::EmptyAuthorDate(state.commit.clone()));
    }
    Ok(git.commit_tree(resolved_tree, std::slice::from_ref(&state.replayed_tip), &info.message, Some(&info.author), None)?)
}
