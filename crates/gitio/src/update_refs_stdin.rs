//! update_refs_stdin: the input for `git update-ref --stdin -z`.

use crate::{GitError, RefUpdate};

/// `start`, one `update` per entry in order, then `prepare` and `commit`, each field NUL-terminated. An empty list
/// gives `start\0prepare\0commit\0`, which git accepts. An empty refname, or one holding a NUL, is refused: a NUL
/// would shift the -z fields and could inject commands.
pub(crate) fn update_refs_stdin(updates: &[RefUpdate]) -> Result<Vec<u8>, GitError> {
    let mut bytes = b"start\0".to_vec();
    for update in updates {
        if update.refname.is_empty() || update.refname.contains('\0') {
            return Err(GitError::refusal(
                Vec::new(),
                format!("not a usable ref name (empty or holding a NUL): {:?}", update.refname),
            ));
        }
        let new = update.new.as_str();
        // An absent ref is written as zeros as long as the new value's hex: on git 2.56, 40 zeros is
        // "invalid <old-oid>" in a sha256 repo and 64 zeros is invalid in a sha1 repo.
        let zeros = "0".repeat(new.len());
        let old = update.old.as_ref().map_or(zeros.as_str(), |oid| oid.as_str());
        for field in ["update ", &update.refname, "\0", new, "\0", old, "\0"] {
            bytes.extend_from_slice(field.as_bytes());
        }
    }
    bytes.extend_from_slice(b"prepare\0commit\0");
    Ok(bytes)
}
