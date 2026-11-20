//! read_squash: a squash commit read back as a Squash. No sibling unit test: it runs git.
//! tests/squash_zit/read_squash_cases.rs covers it in a throwaway repository; squash_from_parts_tests covers the
//! decisions it makes from what it reads.

use crate::squash::{squash_from_parts, Squash, SquashError};
use gitio::{Git, Oid};
use oplog::pick_mortar_trailers;

/// Reads `commit`'s message, its trailers and its request lines, and fills Squash as squash_type documents: its
/// change ID from Mortar-Change-Id, its source from Mortar-Squash-Of, its requests from the message's request
/// lines. Refusals as squash_from_parts makes them; a failed git run is Git.
pub fn read_squash(git: &Git, commit: &Oid) -> Result<Squash, SquashError> {
    let message = git.commit_info(commit).map_err(SquashError::Git)?.message;
    let pairs = git.trailers(&message).map_err(SquashError::Git)?;
    squash_from_parts(commit.clone(), pick_mortar_trailers(&pairs), &pairs, &message)
}
