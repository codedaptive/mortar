//! The report of a `mortar back-out` that land's back_out answered (SPEC-cli 06).

use std::path::Path;

use gitio::Oid;
use land::BackOutAnswer;

use crate::integration_folder::integration_folder_line;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "back-out";

/// - BackedOut: exit 0 "backed-out", integration back at `integration`, the squash and its ref kept with the same
///   change ID, and whether the integration folder moved; next fixes the change in its zone and squashes again.
///   Fields change_id, integration and folder (Null when no folder moved).
/// - NothingToBackOut: exit 0 "nothing-to-back-out", integration already at origin `integration`; fields
///   integration and folder (Null).
pub fn back_out_report(answer: &BackOutAnswer, integration: &Oid, folder: Option<&Path>) -> Report {
    let integration_field = ("integration".to_string(), OutValue::Text(integration.to_string()));
    let (outcome, lines, next, fields) = match answer {
        BackOutAnswer::BackedOut { change_id } => (
            "backed-out",
            vec![
                format!("backed out change {change_id}: integration is back at {integration}"),
                "the squash and its ref are kept, with the same change ID".to_string(),
                integration_folder_line(folder, integration),
            ],
            "fix the change in its zone, then mortar squash <zone> --actor <name>",
            vec![
                ("change_id".to_string(), OutValue::Text(change_id.to_string())),
                integration_field,
                ("folder".to_string(), folder.map_or(OutValue::Null, |path| OutValue::Text(path.display().to_string()))),
            ],
        ),
        BackOutAnswer::NothingToBackOut => (
            "nothing-to-back-out",
            vec![format!("integration is at origin {integration}; there is nothing to back out")],
            "nothing to do",
            vec![integration_field, ("folder".to_string(), OutValue::Null)],
        ),
    };
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next: next.to_string(),
        fields,
    }
}
