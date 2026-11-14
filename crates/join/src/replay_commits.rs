//! replay_commits: replay a list of commits one by one onto a running tip. replay_commits_tests covers only the
//! empty list; replay_fallback_tests covers it through rev_list. Cases with commits run replay_commit underneath, so
//! they are Nagatha's integration tests (rule 11; WO-join-3g order):
//! - a start that is not onto is the first merge's ours and the first commit's parent (SPEC-join §WO-join-1: "onto
//!   the running tip");
//! - a conflict at the first listed commit stops at start with the rest remaining (S-21: "the replayed tip so far,
//!   remaining commits").

use crate::replay_commit::replay_commit;
use crate::replay_git::ReplayGit;
use crate::step::Step;
use crate::{JoinError, ReplayResult};
use gitio::Oid;

/// Replays `commits`, oldest first, each onto the tip replayed so far, starting at `start` and reading merge
/// attributes from `onto`. Clean: the last new commit (`start` for an empty list). At the first conflict it stops:
/// the conflicted commit, the tip replayed before it, the commits after it, and the conflicted tree and files.
/// Writes objects; moves no ref.
pub(crate) fn replay_commits(
    git: &impl ReplayGit,
    onto: &Oid,
    start: &Oid,
    commits: &[Oid],
) -> Result<ReplayResult, JoinError> {
    let mut running_tip = start.clone();
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
