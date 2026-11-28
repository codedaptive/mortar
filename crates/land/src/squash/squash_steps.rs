//! squash_steps: the steps squash_with_message and squash_with_hooks share, with the message to commit decided by
//! the caller's `finish`. It takes no lock: the caller holds `land` and has run recover. It never moves the zone
//! branch and never rewrites a zone commit.
//! No unit test: it runs git and writes the op log; squash_requests_tests, squash_message_tests,
//! choose_change_id_tests, same_squash_tests and squash_applied_tests cover its decisions. In a throwaway
//! repository, tests/squash_zit/hook_cases.rs covers it with hooks, and tests/squash_zit/message_cases.rs,
//! tests/squash_zit/squash_cases.rs and tests/squash_zit/resquash.rs cover it without.

use crate::squash::{
    check_attr_source, choose_change_id, read_prior_squash, read_tip, request_commits, same_squash, squash_applied,
    squash_message, squash_op, squash_ref, squash_requests, squash_trailer_pairs, LandRefs, Squash, SquashAnswer,
    SquashError,
};
use gitio::{Git, MergeTreeOutcome, Oid};
use oplog::{apply, new_change_id, new_op_id, unix_now};

/// Squashes `refs/heads/zone/<zone>` onto the tip of `refs/heads/<origin_branch>`, as squash_with_message
/// describes, except for the message committed. Nothing and Conflicted answer before `finish` is called. Otherwise
/// `finish` gets the merged tree, the composed message (squash_message's text with squash_trailer_pairs' trailers),
/// those trailer pairs in written order and the requests in message order, and answers the message to commit or
/// a refusal. That message is the one same_squash compares with the prior squash (Same, nothing written) and the
/// one committed (Made, one op). Every SquashError becomes E.
pub(crate) fn squash_steps<E: From<SquashError>>(
    git: &Git,
    refs: &LandRefs,
    zone: &str,
    actor: &str,
    message: Option<&str>,
    finish: impl FnOnce(&Oid, String, &[(String, String)], &[String]) -> Result<String, E>,
) -> Result<SquashAnswer, E> {
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
    let trailers = squash_trailer_pairs(&change_id, zone, &source);
    let composed = git.with_trailers(&squash_message(zone, &groups, message), &trailers).map_err(SquashError::Git)?;
    let requests: Vec<String> = groups.into_iter().map(|group| group.request).collect();
    let message = finish(&tree, composed, &trailers, &requests)?;
    if let Some(prior) = prior.as_ref().filter(|prior| same_squash(&prior.info, &tree, &origin, &message)) {
        return Ok(SquashAnswer::Same(Squash { commit: prior.commit.clone(), change_id, source, requests }));
    }
    let commit = git.commit_tree(&tree, &[origin], &message, None, None).map_err(SquashError::Git)?;
    let common = git.common_dir().map_err(SquashError::Git)?;
    let op = squash_op(&refname, prior.as_ref().map(|prior| &prior.commit), &commit, new_op_id(), actor, unix_now());
    squash_applied(apply(git, &common, &op))?;
    Ok(SquashAnswer::Made(Squash { commit, change_id, source, requests }))
}
