//! zone_folder_refusal: a snapshot in a zone folder is refused (exit 3) naming the folder, with a next that names
//! mr snapshot with --actor in a request or attempt folder.

use std::path::Path;

use super::zone_folder_refusal;
use crate::report::{ExitCode, Report};

#[test]
fn a_zone_folder_is_refused_naming_it() {
    assert_eq!(
        zone_folder_refusal(Path::new("/w/zone-core")),
        Report {
            verb: "snapshot".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "mr snapshot runs in a request or attempt folder; /w/zone-core is a zone folder".to_string()
            ],
            owner: None,
            next: "run mr snapshot --actor <name> in a request or attempt folder".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn the_line_names_the_folder_it_was_given() {
    assert_eq!(
        zone_folder_refusal(Path::new("/repo/folders/land")).lines,
        vec!["mr snapshot runs in a request or attempt folder; /repo/folders/land is a zone folder".to_string()]
    );
}
