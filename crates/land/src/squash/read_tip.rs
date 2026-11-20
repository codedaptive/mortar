//! read_tip: the commit a ref land reads stands at.
//! No unit test: it runs git. tests/squash_zit.rs covers it, in a throwaway repository: every squash and
//! would_merge case reads its tips here.

use crate::squash::SquashError;
use gitio::{Git, Oid};

/// The id the full ref `refname` points at; RefMissing naming it when it is absent; a failed git run as Git.
pub(crate) fn read_tip(git: &Git, refname: &str) -> Result<Oid, SquashError> {
    git.read_ref(refname)
        .map_err(SquashError::Git)?
        .ok_or_else(|| SquashError::RefMissing { refname: refname.to_string() })
}
