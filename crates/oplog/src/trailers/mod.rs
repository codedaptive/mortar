//! Mortar's trailers (S-2): Mortar-Change-Id, Mortar-Request and Mortar-Zone, picked from and turned into
//! (key, value) pairs, carried from an old message to a new one, and read and written through gitio.

mod carry_mortar_trailers;
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
mod carry_mortar_trailers_tests;
#[cfg(test)]
mod mortar_trailer_error_display_tests;
#[cfg(test)]
mod mortar_trailer_error_source_tests;
#[cfg(test)]
mod mortar_trailer_pairs_tests;
#[cfg(test)]
mod pick_mortar_trailers_tests;

pub use carry_mortar_trailers::carry_mortar_trailers;
pub use mortar_trailer_error::MortarTrailerError;
pub use mortar_trailer_keys::{MORTAR_CHANGE_ID_KEY, MORTAR_REQUEST_KEY, MORTAR_ZONE_KEY};
pub use mortar_trailer_pairs::mortar_trailer_pairs;
pub use mortar_trailers::MortarTrailers;
pub use pick_mortar_trailers::pick_mortar_trailers;
pub use read_mortar_trailers::read_mortar_trailers;
pub use write_mortar_trailers::write_mortar_trailers;
