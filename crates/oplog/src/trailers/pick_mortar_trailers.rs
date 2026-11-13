//! Picking Mortar's trailers out of a message's (key, value) trailer pairs, as gitio's trailers returns them.

use super::{MortarTrailers, MORTAR_CHANGE_ID_KEY, MORTAR_REQUEST_KEY, MORTAR_ZONE_KEY};
use crate::change_id::{ChangeId, ChangeIdError};

/// Mortar's trailers among `pairs`; a trailer not found is None, and other keys are ignored. Keys match
/// ASCII case-insensitively, as git matches trailer keys. When a key repeats, the last pair wins: Mortar
/// writes its trailers last (S-10), so the last is the one Mortar wrote. Every Mortar-Change-Id value is
/// parsed, earlier ones too, so a malformed one is refused with Malformed naming its text even when a later
/// one is valid: a corrupted ID is never skipped silently. Values are used as given.
pub fn pick_mortar_trailers(pairs: &[(String, String)]) -> Result<MortarTrailers, ChangeIdError> {
    let mut trailers = MortarTrailers::default();
    for (key, value) in pairs {
        if key.eq_ignore_ascii_case(MORTAR_CHANGE_ID_KEY) {
            trailers.change_id = Some(ChangeId::parse(value)?);
        } else if key.eq_ignore_ascii_case(MORTAR_REQUEST_KEY) {
            trailers.request = Some(value.clone());
        } else if key.eq_ignore_ascii_case(MORTAR_ZONE_KEY) {
            trailers.zone = Some(value.clone());
        }
    }
    Ok(trailers)
}
