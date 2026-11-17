//! The words of `mr rereview` after its globals, read into the reviewed squash and the new one.

use gitio::Oid;

/// Exactly two words, the reviewed squash's full commit id then the new one's. The Err is the reason in the cli's
/// own words (Oid::parse's refusal is never printed): fewer than two words, a word after the two, or, naming it, a
/// word that is not a full commit id.
pub fn parse_rereview_args(rest: &[String]) -> Result<(Oid, Oid), String> {
    let [reviewed, new, extra @ ..] = rest else {
        return Err("rereview needs two commits: the reviewed squash and the new one".to_string());
    };
    if let Some(word) = extra.first() {
        return Err(format!("unexpected word {word:?} after the two commits"));
    }
    Ok((commit(reviewed)?, commit(new)?))
}

/// The word as a full commit id, or the reason naming it.
fn commit(word: &str) -> Result<Oid, String> {
    Oid::parse(word).map_err(|_| format!("{word:?} is not a full commit id (40 or 64 lowercase hex characters)"))
}
