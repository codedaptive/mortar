//! hooked_message_reads_back: whether the message the commit hooks left keeps the Mortar trailers and request
//! lines of the squash Mortar composed. squash_with_hooks also parses every Mortar-Change-Id before calling it.
//! hooked_message_reads_back_tests covers it.

use crate::squash::{last_trailer_value, squash_message_requests, HookedSquashError};

/// Ok when every pair of `written` (the trailers squash_trailer_pairs made, in order) is the last value of its key
/// in `hooked_trailers` (git's trailers of `hooked`), and `hooked`'s request lines read back as `requests`.
/// Otherwise Unreadable with `hooked` as its message and, as its part, the first written key whose value is
/// missing or differs, else "requests". Trailers the hooks added after Mortar's are allowed; a later value of a
/// Mortar key is the one the readers take, so it must match. Earlier values of a key are not checked here.
pub(crate) fn hooked_message_reads_back(
    written: &[(String, String)],
    hooked_trailers: &[(String, String)],
    requests: &[String],
    hooked: &str,
) -> Result<(), HookedSquashError> {
    let unreadable = |part: &str| HookedSquashError::Unreadable { part: part.to_string(), message: hooked.to_string() };
    if let Some((key, _)) =
        written.iter().find(|(key, value)| last_trailer_value(hooked_trailers, key) != Some(value.as_str()))
    {
        return Err(unreadable(key));
    }
    match squash_message_requests(hooked) {
        Some(read) if read == requests => Ok(()),
        _ => Err(unreadable("requests")),
    }
}
