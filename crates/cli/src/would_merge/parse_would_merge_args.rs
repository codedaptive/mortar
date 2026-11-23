//! The words of `mr would-merge` after its globals, read into the squash it asks about and the squashes each
//! `--with` adds.

use gitio::Oid;
use oplog::ChangeId;

use super::WouldMergeWords;
use crate::landing::SquashName;

/// `--with <word>` may come anywhere, as often as wanted, kept in order; every other word: the first is the squash,
/// a later one is refused by name. Each squash word is a full commit id (Commit), else a change ID (Change). The Err
/// is the reason, in this order: a second squash word (named), a `--with` with no word after it, no squash word
/// (also when only `--with`s are given), then the first word that is neither (named), the squash's before any
/// `--with`'s. Oid's and ChangeId's own parse errors are never printed.
pub fn parse_would_merge_args(rest: &[String]) -> Result<WouldMergeWords, String> {
    let mut squash = None;
    let mut with = Vec::new();
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        if word == "--with" {
            with.push(words.next().ok_or("--with needs a squash: its commit id or its change ID")?);
        } else if squash.is_some() {
            return Err(format!("unexpected word {word:?} after the squash"));
        } else {
            squash = Some(word);
        }
    }
    let squash = squash.ok_or("would-merge needs a squash: its commit id or its change ID")?;
    Ok(WouldMergeWords {
        squash: squash_name(squash)?,
        with: with.into_iter().map(|word| squash_name(word)).collect::<Result<_, _>>()?,
    })
}

fn squash_name(word: &str) -> Result<SquashName, String> {
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
