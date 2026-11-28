//! squash_with_hooks: squash_with_message's squash with the repository's pre-commit and commit-msg hooks run on
//! the composed message before the repeat check. It takes no lock: the caller holds `land` and has run recover.
//! No unit test: it runs git and the hooks; tests/squash_zit/hook_cases.rs covers it in a throwaway repository,
//! a_commit_msg_hook_that_adds_a_malformed_change_id_is_unreadable_and_moves_nothing among its cases, and
//! hooked_message_reads_back_tests covers its trailer and request comparison.

use crate::squash::{hooked_message_reads_back, squash_steps, HookedSquashError, LandRefs, SquashAnswer, SquashError};
use gitio::{CommitHooksOutcome, Git};
use oplog::{pick_mortar_trailers, MORTAR_CHANGE_ID_KEY};

/// Squashes `refs/heads/zone/<zone>` onto the tip of `refs/heads/<origin_branch>` as squash_with_message does,
/// with the commit hooks run (Git::commit_hooks) on every squash that has a commit to make: Nothing and Conflicted
/// run none; a hook that is missing or not executable is skipped, as Git::commit_hooks skips it. A refusing hook is
/// Refused with nothing written and no op, even on a repeat that would have answered Same. The message the hooks
/// leave must read back as read_squash and read_prior_squash read it: every Mortar-Change-Id a change ID (else
/// Unreadable naming Mortar-Change-Id), the last value of each Mortar trailer written unchanged, and the same
/// requests; else Unreadable with nothing written. That hooked text is what same_squash compares with the prior
/// squash and what is committed, so a commit-msg hook that rewrites the same way each time still answers Same on a
/// repeat. Every other refusal is squash_with_message's, as Squash.
pub fn squash_with_hooks(
    git: &Git,
    refs: &LandRefs,
    zone: &str,
    actor: &str,
    message: Option<&str>,
) -> Result<SquashAnswer, HookedSquashError> {
    squash_steps(git, refs, zone, actor, message, |tree, composed, written, requests| {
        let hooked = match git.commit_hooks(tree, &composed).map_err(SquashError::Git)? {
            CommitHooksOutcome::Refused(refusal) => return Err(HookedSquashError::Refused(refusal)),
            CommitHooksOutcome::Message(hooked) => hooked,
        };
        let pairs = git.trailers(&hooked).map_err(SquashError::Git)?;
        if pick_mortar_trailers(&pairs).is_err() {
            return Err(HookedSquashError::Unreadable { part: MORTAR_CHANGE_ID_KEY.to_string(), message: hooked });
        }
        hooked_message_reads_back(written, &pairs, requests, &hooked)?;
        Ok(hooked)
    })
}
