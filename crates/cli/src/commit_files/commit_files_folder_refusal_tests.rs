//! commit_files_folder_refusal: a zone folder is not refused; a request folder and an attempt folder are, naming the
//! folder and its kind.

use std::path::Path;

use folders::FolderKind;

use super::commit_files_folder_refusal;
use crate::report::{ExitCode, Report};

fn refused(first: &str) -> Option<Report> {
    Some(Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![first.to_string(), "nothing was written and no ref moved".to_string()],
        owner: None,
        next: "mr save \"<sentence>\" saves a request or attempt folder's work, and mr join takes it to the zone"
            .to_string(),
        fields: Vec::new(),
    })
}

#[test]
fn only_a_zone_folder_is_let_through() {
    for (kind, expected) in [
        (FolderKind::Zone, None),
        (
            FolderKind::Request,
            refused("/w/fix-1 is a request folder; commit-files runs in a zone folder or the integration folder"),
        ),
        (
            FolderKind::Attempt(3),
            refused("/w/fix-1 is attempt folder 3; commit-files runs in a zone folder or the integration folder"),
        ),
    ] {
        assert_eq!(commit_files_folder_refusal(Path::new("/w/fix-1"), &kind), expected, "{kind:?}");
    }
}
