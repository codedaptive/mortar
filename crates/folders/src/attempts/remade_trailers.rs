//! The trailers of the commit an orphan attempt is re-made at.

use oplog::MortarTrailers;

/// The trailers of `parent` (its snapshot's first parent; None when they could not be read, as for a malformed
/// change ID) when it carries a change ID, Mortar-Request `request` and Mortar-Zone `zone`, so save finds the
/// request's change ID on HEAD; otherwise none, so save draws a new ID as on a zone tip. Never an attempt trailer.
pub fn remade_trailers(zone: &str, request: &str, parent: Option<&MortarTrailers>) -> MortarTrailers {
    match parent {
        Some(trailers)
            if trailers.change_id.is_some()
                && trailers.request.as_deref() == Some(request)
                && trailers.zone.as_deref() == Some(zone) =>
        {
            trailers.clone()
        }
        _ => MortarTrailers::default(),
    }
}
