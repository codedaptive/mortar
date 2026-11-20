//! (refname, hex) pairs to the (refname, Oid) list a sweep picks.

use gitio::Oid;

use super::oid;

pub(crate) fn picked(pairs: &[(&str, &str)]) -> Vec<(String, Oid)> {
    pairs.iter().map(|(name, hex)| (name.to_string(), oid(hex))).collect()
}
