//! The words of `mr would-merge` after its globals, read into the squash it asks about.

use gitio::Oid;
use oplog::ChangeId;

use crate::landing::SquashName;

/// Exactly one word: a full commit id is Commit, else a change ID is Change. The Err is the reason: no word, a
/// second word (named), or a word that is neither (named). Oid's and ChangeId's own parse errors are never printed.
pub fn parse_would_merge_args(rest: &[String]) -> Result<SquashName, String> {
    let Some((word, words)) = rest.split_first() else {
        return Err("would-merge needs a squash: its commit id or its change ID".to_string());
    };
    if let Some(extra) = words.first() {
        return Err(format!("unexpected word {extra:?} after the squash"));
    }
    if let Ok(commit) = Oid::parse(word) {
        return Ok(SquashName::Commit(commit));
    }
    ChangeId::parse(word).map(SquashName::Change).map_err(|_| {
        format!(
            "{word:?} is neither a full commit id (40 or 64 lowercase hex characters) nor a change ID (32 letters \
             k..z)"
        )
    })
}
