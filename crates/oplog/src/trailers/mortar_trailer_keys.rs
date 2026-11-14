//! The keys of Mortar's four trailers (S-2, S-57). Consts only: pick_mortar_trailers_tests and
//! mortar_trailer_pairs_tests assert the literal text of the first three, and mortar_trailer_keys_tests asserts
//! MORTAR_SQUASH_OF_KEY's (S-7 extension).

/// The trailer carrying the permanent change ID.
pub const MORTAR_CHANGE_ID_KEY: &str = "Mortar-Change-Id";
/// The trailer naming the request a commit belongs to.
pub const MORTAR_REQUEST_KEY: &str = "Mortar-Request";
/// The trailer naming the zone a commit belongs to.
pub const MORTAR_ZONE_KEY: &str = "Mortar-Zone";
/// The trailer naming the zone tip a squash was made from (S-57).
pub const MORTAR_SQUASH_OF_KEY: &str = "Mortar-Squash-Of";
