//! count_chars: the characters in a run of UTF-8 bytes, counted without decoding.

/// The bytes that are not UTF-8 continuation bytes (`10xxxxxx`): one per character in valid UTF-8. Nothing is decoded,
/// so invalid UTF-8 still counts (a stray lead byte counts one, a stray continuation byte none), and a character
/// split across two buffers counts once in their sum, since only its lead byte counts.
pub(crate) fn count_chars(bytes: &[u8]) -> u64 {
    bytes.iter().map(|&b| u64::from(b & 0xC0 != 0x80)).sum()
}
