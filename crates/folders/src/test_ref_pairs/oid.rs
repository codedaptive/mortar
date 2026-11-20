//! Hex to Oid.

use gitio::Oid;

pub(crate) fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}
