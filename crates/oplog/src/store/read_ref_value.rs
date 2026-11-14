//! Reads a ref's current value through git, by its full name.
//! No unit test: it runs git (S-5) and holds no decision. It uses gitio's read_ref (show-ref --verify) because
//! that reads only the full name given; rev-parse --verify would also try short-name lookups and could answer
//! with another ref. A corrupt ref reads as absent: git answers it as it answers a missing one. Nagatha's
//! integration group, in a throwaway repository, should cover: an existing ref returns its id; a missing ref
//! returns None; refs/heads/x absent while refs/heads/refs/heads/x exists reads as absent; a git that cannot run
//! is OplogError::Git.

use gitio::{Git, Oid};

use super::OplogError;

/// The object id `refname` holds, or None when the ref does not exist, or is corrupt: git cannot tell them
/// apart.
pub fn read_ref_value(git: &Git, refname: &str) -> Result<Option<Oid>, OplogError> {
    git.read_ref(refname).map_err(OplogError::Git)
}
