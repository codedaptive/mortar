//! read_path_entries: read the answer of `ls-tree -z --full-tree <from> -- <paths...>`.

use crate::{GitError, Oid};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// One item per path in `paths`, in request order: the mode and id of the NUL-ended record
/// `<mode> <type> <oid>\t<path>` whose path is that path byte for byte, or None when there is none (the tree lacks
/// it). Records are looked up, never consumed: ls-tree prints tree order, and a path asked twice gets its record twice.
/// A `blob` (a file or a 120000 symlink) or `commit` (a 160000 gitlink) is taken; a `tree` is refused naming the path,
/// which must name a file entry. A record for a path not asked, a mode that is not octal digits, a bad id, a missing
/// final NUL or any other shape is refused naming the output.
pub(crate) fn read_path_entries(stdout: &[u8], paths: &[PathBuf]) -> Result<Vec<Option<(String, Oid)>>, GitError> {
    let refuse = |why: &str| {
        GitError::refusal(
            Vec::new(),
            format!("not ls-tree records for the paths asked ({why}): {:?}", String::from_utf8_lossy(stdout)),
        )
    };
    let mut records: Vec<(&[u8], String, Oid)> = Vec::new();
    if !stdout.is_empty() {
        let body = stdout.strip_suffix(b"\0").ok_or_else(|| refuse("no final NUL"))?;
        for record in body.split(|b| *b == 0) {
            records.push(entry(record, paths, &refuse)?);
        }
    }
    Ok(paths
        .iter()
        .map(|path| {
            let wanted = path.as_os_str().as_bytes();
            records.iter().find(|record| record.0 == wanted).map(|record| (record.1.clone(), record.2.clone()))
        })
        .collect())
}

/// The path, mode and id of one record (without its NUL), split at its first tab (the path may hold more) and the
/// header's two spaces.
fn entry<'a>(
    record: &'a [u8],
    paths: &[PathBuf],
    refuse: &dyn Fn(&str) -> GitError,
) -> Result<(&'a [u8], String, Oid), GitError> {
    let tab = record.iter().position(|b| *b == b'\t').ok_or_else(|| refuse("not <mode> <type> <oid>\\t<path>"))?;
    let header: Vec<&[u8]> = record[..tab].split(|b| *b == b' ').collect();
    let [mode, kind, hex] = header[..] else {
        return Err(refuse("not <mode> <type> <oid>\\t<path>"));
    };
    if mode.is_empty() || !mode.iter().all(|b| (b'0'..=b'7').contains(b)) {
        return Err(refuse("a mode that is not octal digits"));
    }
    let oid = std::str::from_utf8(hex)
        .ok()
        .and_then(|hex| Oid::parse(hex).ok())
        .ok_or_else(|| refuse("a bad object id"))?;
    let found = &record[tab + 1..];
    let path = paths
        .iter()
        .find(|path| path.as_os_str().as_bytes() == found)
        .ok_or_else(|| refuse("a path not asked"))?;
    match kind {
        b"blob" | b"commit" => Ok((found, String::from_utf8_lossy(mode).into_owned(), oid)),
        b"tree" => Err(GitError::refusal(Vec::new(), format!("not a file entry: {path:?} is a folder"))),
        _ => Err(refuse("a type that is not blob, commit or tree")),
    }
}
