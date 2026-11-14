//! replay_fallback: replay a range commit by commit, for a git without a usable replay, or to name the conflict
//! git's replay does not.

use crate::replay_commit::replay_commit;
use crate::replay_git::ReplayGit;
use crate::step::Step;
use crate::{JoinError, ReplayResult};
use gitio::Oid;

/// Replays the commits of `base..tip`, oldest first, each onto the tip replayed so far, starting at `onto`. Clean:
/// the last new commit (`onto` for an empty range). At the first conflict it stops: the conflicted commit, the tip
/// replayed before it, the commits after it, and the conflicted tree and files. Writes objects; moves no ref.
pub(crate) fn replay_fallback(
    git: &impl ReplayGit,
    base: &Oid,
    tip: &Oid,
    onto: &Oid,
) -> Result<ReplayResult, JoinError> {
    let commits = git.rev_list(base, tip)?;
    let mut running_tip = onto.clone();
    for (index, commit) in commits.iter().enumerate() {
        match replay_commit(git, onto, &running_tip, commit)? {
            Step::Clean(new) => running_tip = new,
            Step::Conflicted { tree, files } => {
                return Ok(ReplayResult::Conflicted {
                    commit: commit.clone(),
                    replayed_tip: running_tip,
                    remaining: commits[index + 1..].to_vec(),
                    tree,
                    files,
                })
            }
        }
    }
    Ok(ReplayResult::Clean(running_tip))
}
