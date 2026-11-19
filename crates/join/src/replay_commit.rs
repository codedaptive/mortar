//! replay_commit: replay one commit onto the running tip.

use crate::replay_git::ReplayGit;
use crate::step::Step;
use crate::JoinError;
use gitio::{MergeTreeOutcome, Oid};

/// Merges `commit` onto `running_tip` from the commit's one parent, reading merge attributes from `onto`. Clean:
/// writes a commit of the merged tree on `running_tip` with the original message (trailers and change ID kept) and
/// author (its raw date unchanged); the committer is left to git's own identity, as git replay does. A commit that
/// becomes empty is dropped: when the merged tree equals the running tip's tree and the commit changed its parent's
/// tree, no commit is written. A commit already empty at its source (its tree equals its parent's) is kept. git's
/// replay decides empties the same way, so both replay paths give one answer. Conflicted: the tree and its
/// files, and no commit is written. A root or merge commit is refused before any merge, and so is a commit whose
/// author date is empty, since commit_tree cannot hand an empty date back unchanged.
pub(crate) fn replay_commit(
    git: &impl ReplayGit,
    onto: &Oid,
    running_tip: &Oid,
    commit: &Oid,
) -> Result<Step, JoinError> {
    let info = git.commit_info(commit)?;
    let parent = match info.parents.as_slice() {
        [parent] => parent,
        [] => return Err(JoinError::RootCommit(commit.clone())),
        _ => return Err(JoinError::MergeCommit(commit.clone())),
    };
    if info.author.date.as_deref() == Some("") {
        return Err(JoinError::EmptyAuthorDate(commit.clone()));
    }
    let tree = match git.merge_tree(onto, Some(parent), running_tip, commit)? {
        MergeTreeOutcome::Clean(tree) => tree,
        MergeTreeOutcome::Conflicted { tree, files } => return Ok(Step::Conflicted { tree, files }),
    };
    if git.commit_info(running_tip)?.tree == tree && git.commit_info(parent)?.tree != info.tree {
        return Ok(Step::Dropped);
    }
    Ok(Step::Clean(git.commit_tree(&tree, std::slice::from_ref(running_tip), &info.message, Some(&info.author), None)?))
}
