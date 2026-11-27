//! The refusal of `mr folder refresh` on a folder that is not a zone folder: a request or attempt folder is brought up
//! to its zone by `mr join`.

use std::path::Path;

use folders::{FolderKind, FolderRecord};

use super::folder_kind_text;
use crate::report::{ExitCode, Report};

/// None for a Zone record. A Request or Attempt record is exit 3 "refused": a line naming `path` and what the folder
/// is (folder_kind_text) and that mr folder refresh runs on a zone folder, then "nothing changed"; owner "the harness
/// that makes Mortar folders"; next "run mr join in <path> to bring it up to its zone".
pub fn refresh_folder_refusal(path: &Path, record: &FolderRecord) -> Option<Report> {
    if record.kind == FolderKind::Zone {
        return None;
    }
    let path = path.display();
    Some(Report {
        verb: "folder".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("{path} is {}: mr folder refresh runs on a zone folder", folder_kind_text(record)),
            "nothing changed".to_string(),
        ],
        owner: Some("the harness that makes Mortar folders".to_string()),
        next: format!("run mr join in {path} to bring it up to its zone"),
        fields: Vec::new(),
    })
}
