//! content_hash: The hash install records for a file: FNV-1a 64. It detects change; it is no security check.

/// The published FNV-1a 64 offset basis and prime.
const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64 of `bytes` as 16 lowercase hex digits.
pub fn content_hash(bytes: &[u8]) -> String {
    let hash = bytes.iter().fold(OFFSET_BASIS, |hash, &byte| (hash ^ u64::from(byte)).wrapping_mul(PRIME));
    format!("{hash:016x}")
}
