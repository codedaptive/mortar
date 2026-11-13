//! Decoding a change ID back to its 16 bytes, the inverse of from_bytes.

use super::ChangeId;

impl ChangeId {
    /// The 128 bits this ID writes. Every char is k..z (the type's invariant), so b'z' - char is 0..=15.
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        for (byte, pair) in bytes.iter_mut().zip(self.0.as_bytes().chunks_exact(2)) {
            *byte = ((b'z' - pair[0]) << 4) | (b'z' - pair[1]);
        }
        bytes
    }
}
