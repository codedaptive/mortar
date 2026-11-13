//! The change ID type. A bare type (S-7); change_id_from_bytes_tests, change_id_to_bytes_tests and
//! change_id_parse_tests exercise it.

/// A Mortar change ID: 128 random bits written as 32 chars of k..z (S-2), one char per 4 bits. jj's
/// alphabet, so an ID is never mistaken for a hex commit hash. Built only by `from_bytes` or `parse`, so
/// the text is always 32 chars of k..z.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChangeId(pub(super) String);
