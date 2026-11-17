//! The Mortar trailers of one message. A bare type (S-7); pick_mortar_trailers_tests and
//! mortar_trailer_pairs_tests exercise it.

use crate::change_id::ChangeId;

/// Three of Mortar's four trailers on a message (S-2); Mortar-Squash-Of is not held here; None means the trailer is absent.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MortarTrailers {
    /// The Mortar-Change-Id trailer.
    pub change_id: Option<ChangeId>,
    /// The Mortar-Request trailer.
    pub request: Option<String>,
    /// The Mortar-Zone trailer.
    pub zone: Option<String>,
}
