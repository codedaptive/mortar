//! read_unmerged_paths: the paths `ls-files --unmerged -z` lists, each once.

use crate::path_from_bytes::path_from_bytes;
use crate::{GitError, Oid};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// Each NUL-ended record is `<mode> SP <oid> SP <stage> TAB <path>`, one per stage of an unmerged entry, in git's
/// order (by path, then stage, so a path's stages are adjacent). The answer is each path once, in that order, bytes as
/// they are (tabs, spaces, newlines and non-UTF-8 kept: the path is everything after the first tab). Empty output is
/// no paths. A record of any other shape (no final NUL, no tab, not three header fields, an empty mode, a bad object
/// id, a stage other than 1, 2 or 3, an empty path) is refused naming the reason and the output.
pub(crate) fn read_unmerged_paths(stdout: &[u8]) -> Result<Vec<PathBuf>, GitError> {
    if stdout.is_empty() {
        return Ok(Vec::new());
    }
    let refuse = |why: &str| {
        GitError::refusal(
            Vec::new(),
            format!("not an unmerged entry list ({why}): {:?}", String::from_utf8_lossy(stdout)),
        )
    };
    let body = stdout.strip_suffix(b"\0").ok_or_else(|| refuse("no final NUL"))?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for record in body.split(|b| *b == 0) {
        let path = record_path(record).map_err(refuse)?;
        if paths.last().map(|last| last.as_os_str().as_bytes()) != Some(path) {
            paths.push(path_from_bytes(path));
        }
    }
    Ok(paths)
}

/// The path of one record, once its header is checked; Err names what is wrong.
fn record_path(record: &[u8]) -> Result<&[u8], &'static str> {
    let tab = record.iter().position(|b| *b == b'\t').ok_or("a record without a tab")?;
    let (header, path) = (&record[..tab], &record[tab + 1..]);
    let fields: Vec<&[u8]> = header.split(|b| *b == b' ').collect();
    let [mode, oid, stage] = fields[..] else {
        return Err("a header not of three fields");
    };
    if mode.is_empty() {
        return Err("an empty mode");
    }
    std::str::from_utf8(oid).ok().and_then(|hex| Oid::parse(hex).ok()).ok_or("a bad object id")?;
    if !matches!(stage, b"1" | b"2" | b"3") {
        return Err("a stage other than 1, 2 or 3");
    }
    if path.is_empty() {
        return Err("an empty path");
    }
    Ok(path)
}
