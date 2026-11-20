//! Encoding 16 bytes as a change ID: jj's reverse-hex, nibble n written as the char b'z' - n.

use super::ChangeId;

impl ChangeId {
    /// The ID for these 128 bits: two chars per byte, high nibble first; 0 is 'z' and 15 is 'k'.
    pub fn from_bytes(bytes: [u8; 16]) -> ChangeId {
        let text = bytes
            .iter()
            .flat_map(|byte| [byte >> 4, byte & 0x0f])
            .map(|nibble| char::from(b'z' - nibble))
            .collect();
        ChangeId(text)
    }
}
