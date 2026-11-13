//! Mortar's trailers as the (key, value) pairs gitio's with_trailers writes.

use super::{MortarTrailers, MORTAR_CHANGE_ID_KEY, MORTAR_REQUEST_KEY, MORTAR_ZONE_KEY};

/// The present trailers as pairs, in the order Mortar-Change-Id, Mortar-Request, Mortar-Zone; an absent one
/// is skipped.
pub fn mortar_trailer_pairs(trailers: &MortarTrailers) -> Vec<(String, String)> {
    [
        (MORTAR_CHANGE_ID_KEY, trailers.change_id.as_ref().map(|id| id.as_str().to_string())),
        (MORTAR_REQUEST_KEY, trailers.request.clone()),
        (MORTAR_ZONE_KEY, trailers.zone.clone()),
    ]
    .into_iter()
    .filter_map(|(key, value)| value.map(|value| (key.to_string(), value)))
    .collect()
}
