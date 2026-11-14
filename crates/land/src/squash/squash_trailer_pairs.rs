//! squash_trailer_pairs: the trailers of a squash commit (S-57) as the pairs gitio's with_trailers writes.

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
