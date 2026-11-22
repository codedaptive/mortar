//! ZoneOld: where a zone folder stands among the values its zone branch has held.
//! A bare type; find_zone_old_tests exercises each variant.

use gitio::Oid;

/// AtTip: the folder's index holds the tip's tree, so it is where it belongs. From: the first value, newest first,
/// whose tree is the index's tree; the folder is taken as standing on it. NoMatch: no value has that tree, so the
/// index holds staged changes or stands nowhere the branch has been.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneOld {
    AtTip,
    From(Oid),
    NoMatch,
}
