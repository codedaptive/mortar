//! replay_commit: replay one commit onto the running tip.

use crate::replay_git::ReplayGit;
use crate::step::Step;
use crate::JoinError;
use gitio::{MergeTreeOutcome, Oid};

/// Merges `commit` onto `running_tip` from the commit's one parent, reading merge attributes from `onto`. Clean:
/// writes a commit of the merged tree on `running_tip` with the original message (trailers and change ID kept) and
/// author (its raw date unchanged); the committer is left to git's own identity, as git replay does. Conflicted:
/// the tree and its files, and no commit is written. A root or merge commit is refused before any merge, and so is a
/// commit whose author date is empty, since commit_tree cannot hand an empty date back unchanged.
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
    match git.merge_tree(onto, Some(parent), running_tip, commit)? {
        MergeTreeOutcome::Clean(tree) => {
            Ok(Step::Clean(git.commit_tree(&tree, &[running_tip.clone()], &info.message, Some(&info.author), None)?))
        }
        MergeTreeOutcome::Conflicted { tree, files } => Ok(Step::Conflicted { tree, files }),
    }
}
