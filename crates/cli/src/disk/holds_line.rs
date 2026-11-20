//! holds_line: the line `mr disk` prints under a folder's bytes line, saying what the folder holds.

use std::path::Path;

use folders::FolderRecord;

use super::FolderHolds;

/// "<path>: unsaved <t> tracked, <u> untracked; " then "joined to zone <zone>", "not joined to zone <zone>" or, for
/// a zone folder (no joined answer), "a zone folder has no joined answer". Counts and joined only: no verdict on
/// removing the folder.
pub fn holds_line(path: &Path, record: &FolderRecord, holds: &FolderHolds) -> String {
    let joined = match holds.joined {
        Some(true) => format!("joined to zone {}", record.zone),
        Some(false) => format!("not joined to zone {}", record.zone),
        None => "a zone folder has no joined answer".to_string(),
    };
    format!(
        "{}: unsaved {} tracked, {} untracked; {joined}",
        path.display(),
        holds.unsaved.tracked,
        holds.unsaved.untracked
    )
}
