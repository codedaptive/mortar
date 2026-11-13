//! read_status_paths: read the answer of `status --porcelain=v2 -z`.

use crate::path_from_bytes::path_from_bytes;
use crate::text_refusal::text_refusal;
use crate::{GitError, StatusPaths};
use std::path::PathBuf;

/// NUL-ended records, the last NUL required; empty output is a clean folder. `1` (8 fields then the path) is changed;
/// `2` (9 fields then the path, the next record its old path) gives both paths as changed; `u` (10 fields then the
/// path) is conflicted; `?` (then the path) is untracked. The path is the rest of the record, spaces and line breaks
/// kept. An empty record or path, too few fields, or any other tag (`!` ignored, `#` header) is refused naming the
/// record.
pub(crate) fn read_status_paths(stdout: &[u8]) -> Result<StatusPaths, GitError> {
    let mut paths = StatusPaths::default();
    if stdout.is_empty() {
        return Ok(paths);
    }
    let body = stdout
        .strip_suffix(b"\0")
        .ok_or_else(|| text_refusal("status output without its final NUL", &String::from_utf8_lossy(stdout)))?;
    let mut records = body.split(|b| *b == 0);
    while let Some(record) = records.next() {
        match record.get(..2) {
            Some(b"1 ") => paths.changed.push(path_field(record, 9)?),
            Some(b"2 ") => {
                paths.changed.push(path_field(record, 10)?);
                let old = records.next().filter(|old| !old.is_empty()).ok_or_else(|| refusal(record))?;
                paths.changed.push(path_from_bytes(old));
            }
            Some(b"u ") => paths.conflicted.push(path_field(record, 11)?),
            Some(b"? ") => paths.untracked.push(path_field(record, 2)?),
            _ => return Err(refusal(record)),
        }
    }
    Ok(paths)
}

/// The last of `fields` space-separated fields: the path, which keeps any spaces of its own. Refused when the
/// record has fewer fields or the path is empty.
fn path_field(record: &[u8], fields: usize) -> Result<PathBuf, GitError> {
    match record.splitn(fields, |b| *b == b' ').nth(fields - 1) {
        Some(path) if !path.is_empty() => Ok(path_from_bytes(path)),
        _ => Err(refusal(record)),
    }
}

/// A refusal naming the record.
fn refusal(record: &[u8]) -> GitError {
    text_refusal("not a status record", &String::from_utf8_lossy(record))
}
