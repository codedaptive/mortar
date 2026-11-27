//! The report of an `mr back-out` that land's back_out_batch answered.

use std::path::Path;

use gitio::Oid;
use land::BatchBackOutAnswer;
use oplog::ChangeId;

use crate::integration_folder::integration_folder_line;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "back-out";

/// - BackedOut: exit 0 "backed-out", integration back at `integration`, the squashes and their refs kept with the
///   same change IDs, and whether the integration folder moved. A batch of one keeps the single-change lines and
///   next (fix the change in its zone and squash again); a batch of two or more names every change in landing order
///   and its next lands again the squashes that stand. Fields change_id (the newest member's), backed_out (every
///   member's, landing order), integration and folder (Null when no folder moved).
/// - NothingToBackOut: exit 0 "nothing-to-back-out", integration already at origin `integration`; fields
///   integration and folder (Null).
pub fn back_out_report(answer: &BatchBackOutAnswer, integration: &Oid, folder: Option<&Path>) -> Report {
    let integration_field = ("integration".to_string(), OutValue::Text(integration.to_string()));
    let (outcome, lines, next, fields) = match answer {
        BatchBackOutAnswer::BackedOut { change_id, change_ids } => {
            let (first, kept, next) = backed_out_text(change_id, change_ids, integration);
            (
                "backed-out",
                vec![first, kept.to_string(), integration_folder_line(folder, integration)],
                next,
                vec![
                    ("change_id".to_string(), OutValue::Text(change_id.to_string())),
                    ("backed_out".to_string(), OutValue::Texts(change_ids.iter().map(ToString::to_string).collect())),
                    integration_field,
                    (
                        "folder".to_string(),
                        folder.map_or(OutValue::Null, |path| OutValue::Text(path.display().to_string())),
                    ),
                ],
            )
        }
        BatchBackOutAnswer::NothingToBackOut => (
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

/// The first line, the kept line and next of a BackedOut. A batch of one keeps the single-change text; this one
/// condition is where that choice lives.
fn backed_out_text(
    change_id: &ChangeId,
    change_ids: &[ChangeId],
    integration: &Oid,
) -> (String, &'static str, &'static str) {
    if change_ids.len() <= 1 {
        return (
            format!("backed out change {change_id}: integration is back at {integration}"),
            "the squash and its ref are kept, with the same change ID",
            "fix the change in its zone, then mr squash <zone> --actor <name>",
        );
    }
    let ids = change_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
    (
        format!("backed out changes {ids} (in landing order): integration is back at {integration}"),
        "the squashes and their refs are kept, with the same change IDs",
        "land again the squashes that stand, in the same order: mr would-merge <change id> --actor <name> names each \
         one's commit, then mr land <commit> --actor <name>; or fix a change in its zone, then mr squash <zone> \
         --actor <name>",
    )
}
