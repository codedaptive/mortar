//! A short stable key hashed from a list of byte fields.
//! op_key and restore_key call it.

/// FNV-1a, 64 bits: written out here rather than std's DefaultHasher, whose algorithm may change between Rust
/// releases, because a repeat must give the same key whichever mortar build runs it.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// "<prefix>-<16 lowercase hex digits>", the FNV-1a 64 hash of the fields, each field ended by a NUL byte so that no
/// two field lists run together into the same bytes. Same inputs give the same key.
pub fn hash_key(prefix: &str, fields: &[&[u8]]) -> String {
    let hash = fields
        .iter()
        .flat_map(|field| field.iter().chain(std::iter::once(&0u8)))
        .fold(FNV_OFFSET, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME));
    format!("{prefix}-{hash:016x}")
}
