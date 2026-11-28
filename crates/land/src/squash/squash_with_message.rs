//! squash_with_message: a zone's changes squashed onto origin as one commit at `refs/mortar/squash/<zone>`, its
//! message led by the caller's text when there is one. It runs no commit hooks: it is squash_steps committing the
//! composed message as it is. It takes no lock: the caller holds `land` and has run recover. It never moves the
//! zone branch and never rewrites a zone commit.
//! No unit test: it runs git and writes the op log; squash_steps' header names the unit tests of its decisions. In
//! a throwaway repository, tests/squash_zit/message_cases.rs covers
//! a_caller_message_comes_before_the_request_lines_and_reads_back,
//! the_same_message_again_answers_same_and_writes_nothing and
//! a_new_message_on_an_unchanged_squash_is_a_new_commit_with_the_same_change_id; tests/squash_zit/squash_cases.rs
//! covers a_squash_keeps_origins_newer_change_and_adds_only_the_zones, a_conflict_lists_the_files_and_moves_nothing
//! and a_zone_at_origin_has_nothing_to_squash; tests/squash_zit/resquash.rs covers a_resquash_keeps_the_change_id,
//! a_repeated_squash_answers_same_and_writes_nothing and
//! a_squash_after_the_prior_one_is_in_origin_draws_a_new_change_id.

use crate::squash::{squash_steps, LandRefs, SquashAnswer, SquashError};
use gitio::Git;

/// Squashes `refs/heads/zone/<zone>` onto the tip of `refs/heads/<origin_branch>`. The zone tip at its merge base
/// with origin: Nothing. Otherwise the zone's changes since the merge base are merged onto origin, with origin as
/// the attribute source (checked to exist first): conflicted, Conflicted with the files and nothing written but
/// git's tree objects. Clean: a commit of the merged tree with origin as its only parent, its message naming each
/// request's commits and carrying Mortar-Change-Id, Mortar-Zone and Mortar-Squash-Of. `message`, trimmed and not
/// empty, leads the message in place of Mortar's `<zone>: <n> requests` subject; the request lines and trailers
/// follow it as without one. The change ID is the prior squash's while that one is not in origin, else a new one.
/// The commit already at the squash ref with the same tree, parent and message answers Same and writes nothing;
/// otherwise the new commit is written and one op moves the squash ref from the prior commit (or from absent) to
/// it: Made. So a new message on an unchanged squash is a new commit with the same change ID.
pub fn squash_with_message(
    git: &Git,
    refs: &LandRefs,
    zone: &str,
    actor: &str,
    message: Option<&str>,
) -> Result<SquashAnswer, SquashError> {
    squash_steps(git, refs, zone, actor, message, |_, composed, _, _| Ok::<_, SquashError>(composed))
}
