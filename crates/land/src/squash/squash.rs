//! squash: a zone's changes squashed onto origin as one commit at `refs/mortar/squash/<zone>` (SPEC-land
//! 01-squash.md, S-57). It takes no lock: the caller holds `land` and has run recover (S-16). It never moves the zone
//! branch and never rewrites a zone commit.
//! No unit test: it runs git and writes the op log (S-5); squash_requests_tests, squash_message_tests,
//! choose_change_id_tests, same_squash_tests and squash_applied_tests cover its decisions. In a
//! throwaway repository, tests/squash_zit/squash_cases.rs
//! covers a_squash_keeps_origins_newer_change_and_adds_only_the_zones, a_conflict_lists_the_files_and_moves_nothing
//! and a_zone_at_origin_has_nothing_to_squash; tests/squash_zit/resquash.rs covers a_resquash_keeps_the_change_id,
//! a_repeated_squash_answers_same_and_writes_nothing and
//! a_squash_after_the_prior_one_is_in_origin_draws_a_new_change_id.

use crate::squash::{
    check_attr_source, choose_change_id, read_prior_squash, read_tip, request_commits, same_squash, squash_applied,
    squash_message, squash_op, squash_ref, squash_requests, squash_trailer_pairs, LandRefs, Squash, SquashAnswer,
    SquashError,
};
use gitio::{Git, MergeTreeOutcome};
use oplog::{apply, new_change_id, new_op_id, unix_now};

/// Squashes `refs/heads/zone/<zone>` onto the tip of `refs/heads/<origin_branch>`. The zone tip at its merge base
/// with origin: Nothing. Otherwise the zone's changes since the merge base are merged onto origin, with origin as
/// the attribute source (checked to exist first): conflicted, Conflicted with the files and nothing written but
/// git's tree objects. Clean: a commit of the merged tree with origin as its only parent, its message naming each
/// request's commits and carrying Mortar-Change-Id, Mortar-Zone and Mortar-Squash-Of. The change ID is the prior
/// squash's while that one is not in origin, else a new one. The commit already at the squash ref with the same
/// tree, parent and message answers Same and writes nothing; otherwise the new commit is written and one op moves
/// the squash ref from the prior commit (or from absent) to it: Made.
pub fn squash(git: &Git, refs: &LandRefs, zone: &str, actor: &str) -> Result<SquashAnswer, SquashError> {
    let origin = read_tip(git, &format!("refs/heads/{}", refs.origin_branch))?;
    let source = read_tip(git, &format!("refs/heads/zone/{zone}"))?;
    let base = git
        .merge_base(&origin, &source)
        .map_err(SquashError::Git)?
        .ok_or_else(|| SquashError::Unrelated { origin: origin.clone(), source: source.clone() })?;
    if source == base {
        return Ok(SquashAnswer::Nothing { source });
    }
    check_attr_source(git, &origin)?;
    let tree = match git.merge_tree(&origin, Some(&base), &origin, &source).map_err(SquashError::Git)? {
        MergeTreeOutcome::Clean(tree) => tree,
        MergeTreeOutcome::Conflicted { files, .. } => return Ok(SquashAnswer::Conflicted { files }),
    };
    let groups = squash_requests(&request_commits(git, &base, &source)?);
    let refname = squash_ref(zone);
    let prior = read_prior_squash(git, &refname, &origin)?;
    let change_id = match choose_change_id(prior.as_ref()) {
        Some(id) => id,
        None => new_change_id().map_err(SquashError::ChangeId)?,
    };
    let message = git
        .with_trailers(&squash_message(zone, &groups), &squash_trailer_pairs(&change_id, zone, &source))
        .map_err(SquashError::Git)?;
    let requests = groups.into_iter().map(|group| group.request).collect();
    if let Some(prior) = prior.as_ref().filter(|prior| same_squash(&prior.info, &tree, &origin, &message)) {
        return Ok(SquashAnswer::Same(Squash { commit: prior.commit.clone(), change_id, source, requests }));
    }
    let commit = git.commit_tree(&tree, &[origin], &message, None, None).map_err(SquashError::Git)?;
    let common = git.common_dir().map_err(SquashError::Git)?;
    let op = squash_op(&refname, prior.as_ref().map(|prior| &prior.commit), &commit, new_op_id(), actor, unix_now());
    squash_applied(apply(git, &common, &op))?;
    Ok(SquashAnswer::Made(Squash { commit, change_id, source, requests }))
}
