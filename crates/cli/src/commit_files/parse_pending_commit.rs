//! parse_pending_commit: the pending marker's text read back into a PendingCommit.
//! parse_pending_commit_tests covers it.

use std::path::PathBuf;

use gitio::Oid;

use super::PendingCommit;

/// Line 1 is the tip, as an Oid; each line after it is a path, and there is at least one. The final "\n" is
/// optional. None for anything else: an empty text, a first line that is not an Oid, no path line, or an empty
/// path line.
pub fn parse_pending_commit(text: &str) -> Option<PendingCommit> {
    let mut lines = text.strip_suffix('\n').unwrap_or(text).split('\n');
    let tip = Oid::parse(lines.next()?).ok()?;
    let paths: Vec<PathBuf> = lines.map(PathBuf::from).collect();
    if paths.is_empty() || paths.iter().any(|path| path.as_os_str().is_empty()) {
        return None;
    }
    Some(PendingCommit { tip, paths })
}
