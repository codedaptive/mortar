//! replay_mapped_in: replay_mapped's decisions, over any ReplayGit, so they can be unit-tested with a fake.

use crate::check_onto::check_onto;
use crate::replay_fallback::replay_fallback;
use crate::replay_git::ReplayGit;
use crate::{JoinError, MappedReplay};
use gitio::Oid;

/// Checks `onto` exists before any other call, then reads the full ref `tip` (TipMissing when absent), then replays
/// `base..tip` onto `onto` commit by commit, mapping each commit to the running tip after it. It never runs git's
/// replay: that prints only the tip, so it cannot say which commit a dropped one maps to. The commit-by-commit replay
/// sees each commit and drops empties as git's replay does, so its result matches replay's in outcome, trees,
/// messages and authors. Conflicted: only the commits before the conflicted one are mapped. Writes objects; moves no
/// ref.
pub(crate) fn replay_mapped_in(
    git: &impl ReplayGit,
    base: &Oid,
    tip: &str,
    onto: &Oid,
) -> Result<MappedReplay, JoinError> {
    check_onto(git, onto)?;
    let tip_oid = git.read_ref(tip)?.ok_or_else(|| JoinError::TipMissing(tip.to_string()))?;
    replay_fallback(git, base, &tip_oid, onto)
}
