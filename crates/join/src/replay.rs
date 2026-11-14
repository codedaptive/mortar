//! replay: replay a request's commits onto a new base, moving no ref.
//! No unit test: it only joins replay_in to gitio::Git; replay_in_tests cover the decisions. Nagatha's integration
//! tests are to cover it: a clean replay giving the new tip with no ref moved; a conflict naming the commit, tree
//! and files; a `merge=union` file merging clean through `--attr-source` on both paths; a missing onto refused.

use crate::replay_in::replay_in;
use crate::{JoinError, ReplayResult};
use gitio::{Git, Oid};

/// Replays the commits of `base..tip` onto `onto`. `tip` is a full ref name (such as `refs/heads/topic`), read by
/// full name only. Uses git's replay when this git has a usable one; otherwise, to name a conflict, or when git's
/// replay prints no update for the tip, replays commit by commit: merge-tree with `--merge-base=<parent>` and
/// `--attr-source=<onto>`, then commit-tree keeping the message, its trailers and the author. Both paths read merge
/// attributes from `onto`, so it checks first that `onto` exists, before either. Writes objects; moves no ref.
pub fn replay(git: &Git, base: &Oid, tip: &str, onto: &Oid) -> Result<ReplayResult, JoinError> {
    replay_in(git, base, tip, onto)
}
