//! zone_folder_refusal: the report of a `mortar snapshot` run in a zone folder, which has no snapshot ref.

use std::path::Path;

use crate::report::{ExitCode, Report};

/// Exit 1 "refused", naming the zone folder at `folder`; nothing ran. The next names the command in the folders
/// that have a snapshot ref (S-112).
pub fn zone_folder_refusal(folder: &Path) -> Report {
    Report {
        verb: "snapshot".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![format!(
            "mortar snapshot runs in a request or attempt folder; {} is a zone folder",
            folder.display()
        )],
        owner: None,
        next: "run mortar snapshot --actor <name> in a request or attempt folder".to_string(),
        fields: Vec::new(),
    }
}
