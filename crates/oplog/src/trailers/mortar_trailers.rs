//! The Mortar trailers of one message. A bare type (S-7); pick_mortar_trailers_tests,
//! mortar_trailer_pairs_tests and carry_mortar_trailers_tests exercise it.

use crate::change_id::ChangeId;

/// Mortar's three trailers on a message (S-2); None means the trailer is absent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MortarTrailers {
    /// The Mortar-Change-Id trailer.
    pub change_id: Option<ChangeId>,
    /// The Mortar-Request trailer.
    pub request: Option<String>,
    /// The Mortar-Zone trailer.
    pub zone: Option<String>,
}
