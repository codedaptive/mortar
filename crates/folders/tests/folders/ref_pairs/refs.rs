//! (hex, refname) pairs to the (Oid, refname) list git's ref listing gives.

use gitio::Oid;

use super::oid;

pub fn refs(names: &[(&str, &str)]) -> Vec<(Oid, String)> {
    names.iter().map(|(hex, name)| (oid(hex), name.to_string())).collect()
}
