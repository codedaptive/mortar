//! The keys of Mortar's three trailers (S-2). Consts only, so no sibling test: the three names are read
//! together, and pick_mortar_trailers_tests and mortar_trailer_pairs_tests assert their literal text (as S-7
//! does for a bare type).

/// The trailer carrying the permanent change ID.
pub const MORTAR_CHANGE_ID_KEY: &str = "Mortar-Change-Id";
/// The trailer naming the request a commit belongs to.
pub const MORTAR_REQUEST_KEY: &str = "Mortar-Request";
/// The trailer naming the zone a commit belongs to.
pub const MORTAR_ZONE_KEY: &str = "Mortar-Zone";
