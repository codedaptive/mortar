//! squash: a zone's changes squashed onto origin as one commit at `refs/mortar/squash/<zone>`, with Mortar's own
//! message. It takes no lock: the caller holds `land` and has run recover.
//! No unit test: it is squash_with_message with no caller message, and squash_with_message's tests cover it. In a
//! throwaway repository, these call squash itself: tests/squash_zit/squash_cases.rs, tests/squash_zit/resquash.rs
//! and tests/squash_zit/read_squash_cases.rs.

use crate::squash::{squash_with_message, LandRefs, SquashAnswer, SquashError};
use gitio::Git;

/// Squashes `refs/heads/zone/<zone>` onto the tip of `refs/heads/<origin_branch>` with Mortar's own message,
/// subject `<zone>: <n> requests`. The zone tip at its merge base with origin: Nothing. A conflicted merge:
/// Conflicted with the files and nothing written but git's tree objects. Clean: a commit of the merged tree with
/// origin as its only parent, its message naming each request's commits and carrying Mortar-Change-Id, Mortar-Zone
/// and Mortar-Squash-Of. The change ID is the prior squash's while that one is not in origin, else a new one. The
/// commit already at the squash ref with the same tree, parent and message answers Same and writes nothing;
/// otherwise the new commit is written and one op moves the squash ref to it: Made.
pub fn squash(git: &Git, refs: &LandRefs, zone: &str, actor: &str) -> Result<SquashAnswer, SquashError> {
    squash_with_message(git, refs, zone, actor, None)
}
