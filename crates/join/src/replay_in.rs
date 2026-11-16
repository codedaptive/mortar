//! replay_in: replay's decisions, over any ReplayGit, so they can be unit-tested with a fake.

use crate::check_onto::check_onto;
use crate::read_git_replay::read_git_replay;
use crate::replay_fallback::replay_fallback;
use crate::replay_git::ReplayGit;
use crate::{JoinError, ReplayResult};
use gitio::Oid;

/// Checks `onto` exists before any other call, then reads the full ref `tip` (TipMissing when absent). With git's
/// replay available it runs it; an answer holding the tip's update is the new tip. A conflicted answer, or one with
/// no updates (git can replay and print nothing), is replayed again commit by commit, whose answer names the commit,
/// tree and files git's replay does not. Without git's replay, the commit-by-commit replay is the answer.
pub(crate) fn replay_in(git: &impl ReplayGit, base: &Oid, tip: &str, onto: &Oid) -> Result<ReplayResult, JoinError> {
    check_onto(git, onto)?;
    let tip_oid = git.read_ref(tip)?.ok_or_else(|| JoinError::TipMissing(tip.to_string()))?;
    if git.replay_available()? {
        if let Some(new) = read_git_replay(git.replay(onto, base, tip)?, tip)? {
            return Ok(ReplayResult::Clean(new));
        }
    }
    replay_fallback(git, base, &tip_oid, onto).map(|replayed| replayed.result)
}
