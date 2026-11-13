//! Carrying an old message's Mortar trailers onto its new (edited or replayed) message, so the change ID
//! survives edits and replays (S-2). The replay itself is join's.

use super::MortarTrailers;

/// The Mortar trailers to write onto the new message: each field is old's when old has it, else new's.
pub fn carry_mortar_trailers(old: &MortarTrailers, new: &MortarTrailers) -> MortarTrailers {
    MortarTrailers {
        change_id: old.change_id.clone().or_else(|| new.change_id.clone()),
        request: old.request.clone().or_else(|| new.request.clone()),
        zone: old.zone.clone().or_else(|| new.zone.clone()),
    }
}
