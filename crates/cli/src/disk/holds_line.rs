//! holds_line: the line `mr disk` prints under a folder's bytes line, saying what the folder holds.

use std::path::Path;

use folders::FolderRecord;

use super::{FolderHolds, JoinedAnswer};

/// "<path>: unsaved <t> tracked, <u> untracked; " then "joined to zone <zone>", "not joined to zone <zone>", for a
/// zone folder "a zone folder has no joined answer", or, when the joined read failed, "joined unknown: <reason>".
/// Counts and joined only: no verdict on removing the folder.
pub fn holds_line(path: &Path, record: &FolderRecord, holds: &FolderHolds) -> String {
    let joined = match &holds.joined {
        JoinedAnswer::Joined => format!("joined to zone {}", record.zone),
        JoinedAnswer::NotJoined => format!("not joined to zone {}", record.zone),
        JoinedAnswer::ZoneFolder => "a zone folder has no joined answer".to_string(),
        JoinedAnswer::Unknown(reason) => format!("joined unknown: {reason}"),
    };
    format!(
        "{}: unsaved {} tracked, {} untracked; {joined}",
        path.display(),
        holds.unsaved.tracked,
        holds.unsaved.untracked
    )
}
