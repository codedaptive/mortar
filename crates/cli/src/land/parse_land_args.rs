//! The words of `mr land` after its globals, read into the squash commit it lands.

use gitio::Oid;
use oplog::ChangeId;

/// Exactly one word, a full commit id. The Err is the reason: no word, a second word (named), a change ID (named,
/// with would-merge as the way to find its commit, since land takes the reviewed commit only), or any other word
/// (named). Oid's and ChangeId's own parse errors are never printed.
pub fn parse_land_args(rest: &[String]) -> Result<Oid, String> {
    let Some((word, words)) = rest.split_first() else {
        return Err("land needs the squash's commit id".to_string());
    };
    if let Some(extra) = words.first() {
        return Err(format!("unexpected word {extra:?} after the commit"));
    }
    Oid::parse(word).map_err(|_| match ChangeId::parse(word) {
        Ok(_) => format!(
            "{word:?} is a change ID; land takes the squash's commit id only (find it with mr would-merge {word} \
             --actor <name>)"
        ),
        Err(_) => format!("{word:?} is not a full commit id (40 or 64 lowercase hex characters)"),
    })
}
