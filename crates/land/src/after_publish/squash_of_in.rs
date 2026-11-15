//! squash_of_in: the zone tip a squash commit names, read from its trailers. squash_of_in_tests covers it.

use crate::squash::MORTAR_SQUASH_OF_KEY;
use oplog::MORTAR_ZONE_KEY;

/// The Mortar-Squash-Of value when `trailers` also hold a Mortar-Zone equal to `zone`; None for a commit of another
/// zone, one with no Mortar-Zone, or one with no Mortar-Squash-Of. Keys match ASCII case-insensitively, as git
/// matches trailer keys, and when a key repeats the last pair counts: Mortar writes its trailers last (S-10), so
/// the last is the one Mortar wrote, the rule oplog's pick_mortar_trailers follows.
pub(crate) fn squash_of_in<'a>(trailers: &'a [(String, String)], zone: &str) -> Option<&'a str> {
    let value = |key: &str| {
        trailers.iter().rev().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    };
    value(MORTAR_ZONE_KEY).filter(|found| *found == zone)?;
    value(MORTAR_SQUASH_OF_KEY)
}
