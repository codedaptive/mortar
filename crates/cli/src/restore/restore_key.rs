//! The op key of an `mr restore`: --key, or one derived from what the command names (S-115(3)).

use std::path::Path;

use crate::opkey::hash_key;

/// `flag` (--key) when given. Else hash_key's "restore-<16 hex digits>" over the verb, the common dir, the op id
/// and each ref named, in order. restore cannot know the values it will move before it runs, so the key is made of
/// what it names: a plain repeat is the same op, and oplog answers Repeated with the done record's moves.
pub fn restore_key(flag: Option<&str>, common: &Path, op_id: &str, refs: &[String]) -> String {
    if let Some(key) = flag {
        return key.to_string();
    }
    let mut fields: Vec<&[u8]> = vec![b"restore", common.as_os_str().as_encoded_bytes(), op_id.as_bytes()];
    fields.extend(refs.iter().map(String::as_bytes));
    hash_key("restore", &fields)
}
