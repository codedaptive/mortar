//! Git::replay: replay a branch's commits onto a new base, moving no ref. It only runs git, so it has no unit tests;
//! Nagatha's integration test "replay giving the update for a clean replay with the ref unmoved, Conflicted for a
//! conflict, and an error for a bad range" is to cover it.

use crate::read_replay::read_replay;
use crate::replay_request::replay_request;
use crate::{Git, GitError, Oid, ReplayOutcome};

impl Git {
    /// Replays the commits of `base..tip_ref` onto `onto`, reading merge attributes from `attr_source`, and returns the
    /// ref moves that would finish it (pass them to `update_refs`), or Conflicted. `tip_ref` must be a ref name such as
    /// `refs/heads/topic`. Writes commits; moves no ref.
    pub fn replay(&self, attr_source: &Oid, onto: &Oid, base: &Oid, tip_ref: &str) -> Result<ReplayOutcome, GitError> {
        let (args, opts) = replay_request(attr_source, onto, base, tip_ref)?;
        read_replay(&self.run(&args, &opts)?)
    }
}
