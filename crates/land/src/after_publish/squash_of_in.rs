//! squash_of_in: the zone tip a squash commit names, read from its trailers. squash_of_in_tests covers it.

use crate::squash::{last_trailer_value, MORTAR_SQUASH_OF_KEY};
use oplog::MORTAR_ZONE_KEY;

/// The Mortar-Squash-Of value when `trailers` also hold a Mortar-Zone equal to `zone`; None for a commit of another
/// zone, one with no Mortar-Zone, or one with no Mortar-Squash-Of. Both keys are read through last_trailer_value:
/// keys match ASCII case-insensitively, as git matches trailer keys, and when a key repeats the last pair counts.
pub(crate) fn squash_of_in<'a>(trailers: &'a [(String, String)], zone: &str) -> Option<&'a str> {
    last_trailer_value(trailers, MORTAR_ZONE_KEY).filter(|found| *found == zone)?;
    last_trailer_value(trailers, MORTAR_SQUASH_OF_KEY)
}
