//! read_prior_squash: the commit already at a zone's squash ref, as choose_change_id and same_squash read it.
//! No unit test: it runs git (S-5); choose_change_id_tests and same_squash_tests cover the decisions made from it.
//! Nagatha's integration tests of SPEC-land 01-squash.md §Tests are to cover it, in a throwaway repository: a
//! re-squash keeps the change ID; a squash after the prior one landed in origin draws a new one.

use crate::squash::{PriorSquash, SquashError};
use gitio::{Git, Oid};
use oplog::read_mortar_trailers;

/// None when `squash_ref` is absent. Otherwise its commit, what the commit holds, its Mortar-Change-Id and whether
/// it is an ancestor of `origin` (its merge base with origin is itself). A failed git run is Git; unreadable
/// Mortar trailers are Trailers.
pub(crate) fn read_prior_squash(git: &Git, squash_ref: &str, origin: &Oid) -> Result<Option<PriorSquash>, SquashError> {
    let Some(commit) = git.read_ref(squash_ref).map_err(SquashError::Git)? else { return Ok(None) };
    let info = git.commit_info(&commit).map_err(SquashError::Git)?;
    let change_id = read_mortar_trailers(git, &info.message).map_err(SquashError::Trailers)?.change_id;
    let in_origin = git.merge_base(&commit, origin).map_err(SquashError::Git)?.as_ref() == Some(&commit);
    Ok(Some(PriorSquash { commit, info, change_id, in_origin }))
}
