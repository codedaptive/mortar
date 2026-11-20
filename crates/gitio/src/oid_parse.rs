//! Oid::parse: validate an object id.

use crate::{GitError, Oid};

impl Oid {
    /// Accepts exactly 40 or 64 chars of `[0-9a-f]`; no trimming, uppercase refused. The refusal names the text.
    pub fn parse(s: &str) -> Result<Oid, GitError> {
        let hex = s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
        if hex && (s.len() == 40 || s.len() == 64) {
            return Ok(Oid(s.to_string()));
        }
        Err(GitError::refusal(
            Vec::new(),
            format!("not a git object id (40 or 64 lowercase hex chars): {s:?}"),
        ))
    }
}
