//! Reads a ref's current value through git, by its full name.
//! No unit test: it runs git and holds no decision. It uses gitio's read_ref (show-ref --verify) because
//! that reads only the full name given; rev-parse --verify would also try short-name lookups and could answer
//! with another ref. A corrupt ref reads as absent: git answers it as it answers a missing one. Covered by
//! the crate's integration tests, through apply and recover: an existing ref returns its id
//! (tests/oplog/apply.rs); a missing ref returns None (tests/oplog/refined_recovery.rs). Integration tests,
//! in a throwaway repository, should cover: refs/heads/x absent while refs/heads/refs/heads/x exists reads as
//! absent; a git that cannot run is OplogError::Git.

use gitio::{Git, Oid};

use super::OplogError;

/// The object id `refname` holds, or None when the ref does not exist, or is corrupt: git cannot tell them
/// apart.
pub fn read_ref_value(git: &Git, refname: &str) -> Result<Option<Oid>, OplogError> {
    git.read_ref(refname).map_err(OplogError::Git)
}
