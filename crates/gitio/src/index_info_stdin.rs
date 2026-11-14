//! index_info_stdin: the input for `git update-index -z --index-info`.

use crate::Oid;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// For each path with its entry, `<mode> <oid>\t<path>\0`; for a path with None, `0 <zeros>\t<path>\0`, which
/// update-index reads as removing the path. git reads the id field as exactly the repo's hash length, so the zeros
/// are `hex_len` long, as update_refs_stdin does for an absent ref. Path bytes go in raw: -z ends each record with a
/// NUL, so line breaks and tabs in a path pass. `paths` and `entries` pair up item by item.
pub(crate) fn index_info_stdin(paths: &[PathBuf], entries: &[Option<(String, Oid)>], hex_len: usize) -> Vec<u8> {
    let zeros = "0".repeat(hex_len);
    let mut bytes = Vec::new();
    for (path, entry) in paths.iter().zip(entries) {
        let (mode, oid) = entry.as_ref().map_or(("0", zeros.as_str()), |(mode, oid)| (mode.as_str(), oid.as_str()));
        for field in [mode.as_bytes(), b" ", oid.as_bytes(), b"\t", path.as_os_str().as_bytes(), b"\0"] {
            bytes.extend_from_slice(field);
        }
    }
    bytes
}
