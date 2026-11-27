//! The report of an `mr folder zone` or `mr folder request` that made (or found already made) its folder.

use std::path::Path;

use folders::{FolderKind, FolderRecord};

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0, outcome "made". Lines: the folder's kind, zone and request and its path; its base (full commit id); its
/// cone (the work order's paths, then the needed paths). Fields: path, kind ("zone", "request" or "attempt"), zone,
/// request (null for a zone folder), base, paths. Next for `sub` "zone": make request folders with mr folder
/// request in that zone; for any other sub: start the request's agent in `path`. `mr folder request` adds to this
/// report: with --with, request_base's partners_line line after these and a field partners after paths; then
/// build_output_lines' lines, and a last field build_output (see run_folder_request).
pub fn folder_made_report(sub: &str, path: &Path, record: &FolderRecord) -> Report {
    let (zone, request) = (&record.zone, record.request.as_deref());
    let (kind, what) = match (record.kind, request) {
        (FolderKind::Zone, _) => ("zone", format!("the zone folder of zone {zone}")),
        (FolderKind::Request, Some(request)) => {
            ("request", format!("the request folder of request {request} in zone {zone}"))
        }
        (FolderKind::Request, None) => ("request", format!("a request folder in zone {zone}")),
        (FolderKind::Attempt(k), _) => ("attempt", format!("attempt folder {k} in zone {zone}")),
    };
    let cone: Vec<String> = record.work_order_paths.iter().chain(&record.needed_paths).cloned().collect();
    let next = if sub == "zone" {
        format!(
            "make request folders with mr folder request {zone} <request> <path> --paths <path>... --actor <name>"
        )
    } else {
        format!("start the request's agent in {}", path.display())
    };
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Done,
        outcome: "made".to_string(),
        lines: vec![
            format!("{what} is at {}", path.display()),
            format!("base {}", record.base.as_str()),
            format!("cone {}", cone.join(", ")),
        ],
        owner: None,
        next,
        fields: vec![
            ("path".to_string(), OutValue::Text(path.display().to_string())),
            ("kind".to_string(), OutValue::Text(kind.to_string())),
            ("zone".to_string(), OutValue::Text(zone.clone())),
            ("request".to_string(), request.map_or(OutValue::Null, |name| OutValue::Text(name.to_string()))),
            ("base".to_string(), OutValue::Text(record.base.as_str().to_string())),
            ("paths".to_string(), OutValue::Texts(cone)),
        ],
    }
}
