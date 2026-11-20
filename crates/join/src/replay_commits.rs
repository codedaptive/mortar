//! replay_commits: replay a list of commits one by one onto a running tip. The loop's decisions (mapping, drops,
//! the conflicted stop) are map_steps_tests'; replay_commits_tests covers only the empty list. Cases with commits run
//! replay_commit underneath, so they are ZITs (tests/replay.rs, tests/replay_mapped.rs):
//! - a start that is not onto is the first merge's ours and the first commit's parent, since each commit is replayed
//!   onto the running tip;
//! - a conflict at the first listed commit stops at start, the tip replayed so far, with the rest of the commits
//!   remaining.

use crate::map_steps::map_steps;
use crate::replay_commit::replay_commit;
use crate::replay_git::ReplayGit;
use crate::{JoinError, MappedReplay};
use gitio::Oid;

/// Replays `commits`, oldest first, each onto the tip replayed so far, starting at `start` and reading merge
/// attributes from `onto`; a commit that becomes empty is dropped (replay_commit). Clean: the running tip after the
/// last commit (`start` for an empty list or when every commit dropped), with each commit mapped to the running tip
/// after it. At the first conflict it stops: the conflicted commit, the tip replayed before it, the commits after
/// it, and the conflicted tree and files, with only the commits before it mapped. Writes objects; moves no ref.
pub(crate) fn replay_commits(
    git: &impl ReplayGit,
    onto: &Oid,
    start: &Oid,
    commits: &[Oid],
) -> Result<MappedReplay, JoinError> {
    map_steps(start, commits, |running, commit| replay_commit(git, onto, running, commit))
}
