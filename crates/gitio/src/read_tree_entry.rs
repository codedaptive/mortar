//! read_tree_entry: read the answer of `ls-tree -z --full-tree <tree> -- <path>`.

use crate::{GitError, Oid};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// Empty output is a path the tree lacks (None). Otherwise exactly one NUL-ended record `<mode> <type> <oid>\t<path>`
/// whose path is `path` byte for byte (it may hold newlines and tabs): a `blob` (a file or a symlink) is its id; any
/// other type is refused as not a blob, naming the type and the path. Two records, another path (a folder path
/// ending "/" lists the folder's children), a bad id or any other shape is refused naming the output.
pub(crate) fn read_tree_entry(stdout: &[u8], path: &Path) -> Result<Option<Oid>, GitError> {
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
    if kind != b"blob" {
        return Err(GitError::refusal(
            Vec::new(),
            format!("not a blob: {path:?} is a {:?}", String::from_utf8_lossy(kind)),
        ));
    }
    Ok(Some(oid))
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
