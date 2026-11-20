//! PatchId::parse: validate a patch ID.

use crate::{GitError, Oid, PatchId};

impl PatchId {
    /// Accepts exactly 40 or 64 chars of `[0-9a-f]`, the rule Oid::parse checks; no trimming, uppercase refused.
    /// The refusal names the text.
    pub fn parse(s: &str) -> Result<PatchId, GitError> {
        Oid::parse(s).map(|oid| PatchId(oid.0)).map_err(|_| {
            GitError::refusal(Vec::new(), format!("not a patch id (40 or 64 lowercase hex chars): {s:?}"))
        })
    }
}
