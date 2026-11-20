//! replay_fallback: replay a range commit by commit, for a git without a usable replay, or to name the conflict
//! git's replay does not.

use crate::replay_commits::replay_commits;
use crate::replay_git::ReplayGit;
use crate::{JoinError, MappedReplay};
use gitio::Oid;

/// Replays the commits of `base..tip`, oldest first, each onto the tip replayed so far, starting at `onto`; a commit
/// that becomes empty is dropped, as git's replay drops it. Clean: the running tip after the last commit (`onto` for
/// an empty range or when every commit dropped), with each commit mapped to the running tip after it. At the first
/// conflict it stops: the conflicted commit, the tip replayed before it, the commits after it, and the conflicted
/// tree and files, with only the commits before it mapped. Writes objects; moves no ref.
pub(crate) fn replay_fallback(
    git: &impl ReplayGit,
    base: &Oid,
    tip: &Oid,
    onto: &Oid,
) -> Result<MappedReplay, JoinError> {
    replay_commits(git, onto, onto, &git.rev_list(base, tip)?)
}
