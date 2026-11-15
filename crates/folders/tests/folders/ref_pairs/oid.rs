//! Hex to Oid.

use gitio::Oid;

pub fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}
