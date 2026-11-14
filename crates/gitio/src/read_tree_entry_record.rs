//! read_tree_entry_record: check the answer of `ls-tree -z --full-tree <tree> -- <path>` is the one record for `path`.

use crate::{GitError, Oid};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Empty output is a path the tree lacks (None). Otherwise the record's type bytes, as found, and its id, after these
/// checks in this order: exactly one NUL-ended record `<mode> <type> <oid>\t<path>` with no NUL inside; its fields;
/// its path is `path` byte for byte (it may hold newlines and tabs; a folder path ending "/" lists the folder's
/// children, so it fails here); a valid id. Any failure is a GitError with no args and no code whose stderr names
/// the reason and the output. The type is not judged here: read_tree_entry and read_path_kind read their answers
/// through this and decide what each type means.
pub(crate) fn read_tree_entry_record<'a>(stdout: &'a [u8], path: &Path) -> Result<Option<(&'a [u8], Oid)>, GitError> {
    if stdout.is_empty() {
        return Ok(None);
    }
    let refuse = |why: &str| {
        GitError::refusal(
            Vec::new(),
            format!("not one ls-tree record for {path:?} ({why}): {:?}", String::from_utf8_lossy(stdout)),
        )
    };
    let record = stdout
        .strip_suffix(b"\0")
        .filter(|record| !record.contains(&0))
        .ok_or_else(|| refuse("not exactly one NUL-ended record"))?;
    let (kind, hex, found) = fields(record).ok_or_else(|| refuse("not <mode> <type> <oid>\\t<path>"))?;
    if found != path.as_os_str().as_bytes() {
        return Err(refuse("another path"));
    }
    let oid = std::str::from_utf8(hex)
        .ok()
        .and_then(|hex| Oid::parse(hex).ok())
        .ok_or_else(|| refuse("a bad object id"))?;
    Ok(Some((kind, oid)))
}

/// The type, id and path of a record, split at its first tab (the path may hold more) and the header's two spaces.
fn fields(record: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    let tab = record.iter().position(|b| *b == b'\t')?;
    let mut header = record[..tab].split(|b| *b == b' ');
    let (mode, kind, hex) = (header.next()?, header.next()?, header.next()?);
    if header.next().is_some() || mode.is_empty() || kind.is_empty() {
        return None;
    }
    Some((kind, hex, &record[tab + 1..]))
}
