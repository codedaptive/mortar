//! request_commits: the subject and Mortar-Request of each commit a squash takes in, as squash_requests reads them.
//! No unit test: it runs git (S-5); commit_subject_tests and squash_requests_tests cover what is made of its answer.
//! Nagatha's integration tests of SPEC-land 01-squash.md §Tests are to cover it, in a throwaway repository: the
//! squash message names each request's commit subjects, grouped by request, in order.

use crate::squash::{commit_subject, SquashError};
use gitio::{Git, Oid};
use oplog::pick_mortar_trailers;

/// (subject, Mortar-Request value) for each commit reachable from `source` and not from `base`, oldest first; None
/// for a commit with no Mortar-Request. A failed git run is Git; a malformed Mortar-Change-Id is ChangeId.
pub(crate) fn request_commits(
    git: &Git,
    base: &Oid,
    source: &Oid,
) -> Result<Vec<(String, Option<String>)>, SquashError> {
    git.rev_list(base, source)
        .map_err(SquashError::Git)?
        .iter()
        .map(|commit| {
            let message = git.commit_info(commit).map_err(SquashError::Git)?.message;
            let pairs = git.trailers(&message).map_err(SquashError::Git)?;
            let request = pick_mortar_trailers(&pairs).map_err(SquashError::ChangeId)?.request;
            Ok((commit_subject(&message), request))
        })
        .collect()
}
