//! replay_fallback: replay a range commit by commit, for a git without a usable replay, or to name the conflict
//! git's replay does not.

use crate::replay_commits::replay_commits;
use crate::replay_git::ReplayGit;
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
    replay_commits(git, onto, onto, &git.rev_list(base, tip)?)
}
