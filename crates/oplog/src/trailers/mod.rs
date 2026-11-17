//! Mortar's trailers (S-2, S-57): Mortar-Change-Id, Mortar-Request, Mortar-Zone and Mortar-Squash-Of. The
//! first three are picked from and turned into (key, value) pairs, and read and written
//! through gitio; Mortar-Squash-Of is a key only here.

mod mortar_trailer_error;
mod mortar_trailer_error_display;
mod mortar_trailer_error_source;
mod mortar_trailer_keys;
mod mortar_trailer_pairs;
mod mortar_trailers;
mod pick_mortar_trailers;
mod read_mortar_trailers;
mod write_mortar_trailers;

#[cfg(test)]
mod mortar_trailer_error_display_tests;
#[cfg(test)]
mod mortar_trailer_error_source_tests;
#[cfg(test)]
mod mortar_trailer_keys_tests;
#[cfg(test)]
mod mortar_trailer_pairs_tests;
#[cfg(test)]
mod pick_mortar_trailers_tests;

pub use mortar_trailer_error::MortarTrailerError;
pub use mortar_trailer_keys::{MORTAR_CHANGE_ID_KEY, MORTAR_SQUASH_OF_KEY, MORTAR_ZONE_KEY};
pub use mortar_trailer_pairs::mortar_trailer_pairs;
pub use mortar_trailers::MortarTrailers;
pub use pick_mortar_trailers::pick_mortar_trailers;
pub use read_mortar_trailers::read_mortar_trailers;
pub use write_mortar_trailers::write_mortar_trailers;

pub(crate) use mortar_trailer_keys::MORTAR_REQUEST_KEY;
