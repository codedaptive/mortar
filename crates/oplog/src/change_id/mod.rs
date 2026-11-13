//! Change IDs: the 32-letter k..z form (S-2), its encoding from 16 bytes and back, parsing, and drawing a
//! new one from /dev/urandom.

mod change_id;
mod change_id_as_str;
mod change_id_display;
mod change_id_error;
mod change_id_error_display;
mod change_id_error_source;
mod change_id_from_bytes;
mod change_id_parse;
mod change_id_to_bytes;
mod new_change_id;

#[cfg(test)]
mod change_id_as_str_tests;
#[cfg(test)]
mod change_id_display_tests;
#[cfg(test)]
mod change_id_error_display_tests;
#[cfg(test)]
mod change_id_error_source_tests;
#[cfg(test)]
mod change_id_from_bytes_tests;
#[cfg(test)]
mod change_id_parse_tests;
#[cfg(test)]
mod change_id_to_bytes_tests;

pub use change_id::ChangeId;
pub use change_id_error::ChangeIdError;
pub use new_change_id::new_change_id;
