//! The refusal of a `mortar folder` sub-verb that runs in a request folder (attempts, keep, drop) when the current
//! folder is a zone or attempt folder.

use folders::{FolderKind, FolderRecord};

use crate::report::{ExitCode, Report};

/// None for a Request record. A Zone or Attempt(k) record is exit 1 "refused": a line naming the folder's kind and
/// zone (for an attempt, also k and its request), then "nothing ran"; owner "the harness that makes Mortar
/// folders"; next "run mortar <verb_line> --actor <name> in the request's own folder". `verb_line` is the verb and
/// sub-verb, e.g. "folder keep".
pub fn request_folder_refusal(verb_line: &str, record: &FolderRecord) -> Option<Report> {
    let zone = &record.zone;
    let what = match (record.kind, record.request.as_deref()) {
        (FolderKind::Request, _) => return None,
        (FolderKind::Zone, _) => format!("this is the zone folder of zone {zone}"),
        (FolderKind::Attempt(k), Some(request)) => {
            format!("this is attempt folder {k} of request {request} in zone {zone}")
        }
        (FolderKind::Attempt(k), None) => format!("this is attempt folder {k} in zone {zone}"),
    };
    Some(Report {
        verb: "folder".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![format!("{what}: mortar {verb_line} runs in a request folder"), "nothing ran".to_string()],
        owner: Some("the harness that makes Mortar folders".to_string()),
        next: format!("run mortar {verb_line} --actor <name> in the request's own folder"),
        fields: Vec::new(),
    })
}
