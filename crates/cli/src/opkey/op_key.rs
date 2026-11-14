//! The op key a verb passes to the op log and the lock (spec: Operation key).
//! Shell API with no caller yet: the ref-moving verbs of WO-cli-2..6 call it.

use std::path::Path;

use oplog::RefMove;

/// FNV-1a, 64 bits: written out here rather than std's DefaultHasher, whose algorithm may change between Rust
/// releases, because a repeat must give the same key whichever mortar build runs it.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// `flag` (--key) when given. Else "<verb>-<16 hex digits>", hashed from the verb, the folder and each move's
/// ref, old value ("none" when the ref must not exist) and new value, each field ended by a NUL byte so that no
/// two field lists run together into the same bytes. Same inputs give the same key, so a plain repeat of a
/// command is the same operation.
pub fn op_key(flag: Option<&str>, verb: &str, folder: &Path, moves: &[RefMove]) -> String {
    if let Some(key) = flag {
        return key.to_string();
    }
    let mut fields: Vec<&[u8]> = vec![verb.as_bytes(), folder.as_os_str().as_encoded_bytes()];
    for one in moves {
        fields.push(one.refname.as_bytes());
        fields.push(one.old.as_ref().map_or("none", |old| old.as_str()).as_bytes());
        fields.push(one.new.as_str().as_bytes());
    }
    let hash = fields
        .iter()
        .flat_map(|field| field.iter().chain(std::iter::once(&0u8)))
        .fold(FNV_OFFSET, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME));
    format!("{verb}-{hash:016x}")
}
