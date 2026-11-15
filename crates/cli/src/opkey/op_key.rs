//! The op key a verb passes to the op log and the lock (spec: Operation key).
//! save's key goes to the lock holder only (S-90).
//! Shell API: `mortar join` (run_join), `mortar need` (run_need) and `mortar save` (run_save) call it, and so will
//! each later verb that moves a ref.

use std::path::Path;

use oplog::RefMove;

use super::hash_key;

/// `flag` (--key) when given. Else hash_key's "<verb>-<16 hex digits>" over the verb, the folder and each move's
/// ref, old value ("none" when the ref must not exist) and new value. Same inputs give the same key, so a plain
/// repeat of a command is the same operation.
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
    hash_key(verb, &fields)
}
