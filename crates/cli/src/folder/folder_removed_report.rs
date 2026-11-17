//! The report of an `mr folder remove` that removed a request folder.

use std::path::Path;

use folders::FolderRecord;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0, outcome "removed". Lines: the request folder at `path` (its request and zone from `record`) is removed,
/// with its branch and its snapshot ref. Fields: path, zone, request (null when the record names none). Next:
/// "nothing more for this request".
pub fn folder_removed_report(path: &Path, record: &FolderRecord) -> Report {
    let zone = &record.zone;
    let what = match record.request.as_deref() {
        Some(request) => format!("the request folder of request {request} in zone {zone}"),
        None => format!("the folder in zone {zone}"),
    };
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Done,
        outcome: "removed".to_string(),
        lines: vec![format!("removed {what} at {}, with its branch and its snapshot ref", path.display())],
        owner: None,
        next: "nothing more for this request".to_string(),
        fields: vec![
            ("path".to_string(), OutValue::Text(path.display().to_string())),
            ("zone".to_string(), OutValue::Text(zone.clone())),
            ("request".to_string(), record.request.clone().map_or(OutValue::Null, OutValue::Text)),
        ],
    }
}
