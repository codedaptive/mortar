//! The refusal of `mr commit-files` in a request or attempt folder: it runs in a zone folder or the integration
//! folder. commit_files_folder_refusal_tests covers it.

use std::path::Path;

use folders::FolderKind;

use crate::report::{ExitCode, Report};

/// None for a zone folder. A request folder or attempt folder n is exit 1 "refused", no owner and no fields: a line
/// naming the folder and its kind, then that nothing was written; next says mr save and mr join carry such a
/// folder's work to its zone.
pub fn commit_files_folder_refusal(folder: &Path, kind: &FolderKind) -> Option<Report> {
    let what = match kind {
        FolderKind::Zone => return None,
        FolderKind::Request => "a request folder".to_string(),
        FolderKind::Attempt(n) => format!("attempt folder {n}"),
    };
    Some(Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!(
                "{} is {what}; commit-files runs in a zone folder or the integration folder",
                folder.display()
            ),
            "nothing was written and no ref moved".to_string(),
        ],
        owner: None,
        next: "mr save \"<sentence>\" saves a request or attempt folder's work, and mr join takes it to the zone"
            .to_string(),
        fields: Vec::new(),
    })
}
