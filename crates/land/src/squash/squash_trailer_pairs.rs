//! squash_trailer_pairs: the trailers of a squash commit as the pairs gitio's with_trailers writes.
//! No sibling unit test: every test of it runs oplog's mortar_trailer_pairs for real.
//! tests/squash_zit/squash_cases.rs a_squash_keeps_origins_newer_change_and_adds_only_the_zones and
//! tests/land_rit/trailers.rs
//! rit_squash_writes_change_id_zone_and_squash_of_in_that_exact_order_without_a_request_trailer assert all three
//! trailers in order.

use crate::squash::MORTAR_SQUASH_OF_KEY;
use gitio::Oid;
use oplog::{mortar_trailer_pairs, ChangeId, MortarTrailers};

/// Mortar-Change-Id, Mortar-Zone and Mortar-Squash-Of (the zone tip squashed), in that order. A squash carries
/// no Mortar-Request: it holds many requests, which its message lines name.
pub(crate) fn squash_trailer_pairs(change_id: &ChangeId, zone: &str, source: &Oid) -> Vec<(String, String)> {
    let trailers = MortarTrailers { change_id: Some(change_id.clone()), request: None, zone: Some(zone.to_string()) };
    let mut pairs = mortar_trailer_pairs(&trailers);
    pairs.push((MORTAR_SQUASH_OF_KEY.to_string(), source.to_string()));
    pairs
}
