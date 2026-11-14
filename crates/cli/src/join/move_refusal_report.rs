//! The report of a folder move that folders::move_folder refused during `mortar join`.

use folders::FolderError;

use crate::report::{failure_report, ExitCode, Report};

const VERB: &str = "join";
const OWNER: &str = "this request's folder";

/// `before` (what the join already did) comes first. Files in the way (MoveRefused) are exit 1 "refused": they
/// are listed, and the next join finishes the move once they are out of the way, since the request branch already
/// stands at the new tip. A branch at neither tip (BranchElsewhere) is exit 1 "refused" with the error's text.
/// Anything else is failure_report with `before` ahead of its lines.
pub fn move_refusal_report(error: &FolderError, before: Vec<String>) -> Report {
    let mut lines = before;
    let next = match error {
        FolderError::MoveRefused { paths, .. } => {
            lines.push("the folder could not move; these files stand in the way:".to_string());
            lines.extend(paths.iter().map(|path| format!("  {}", path.display())));
            lines.push(
                "the request branch already stands at the new tip; the next mortar join finishes the move".to_string(),
            );
            "move those files out of the way, then mortar join"
        }
        FolderError::BranchElsewhere { .. } => {
            lines.push(error.to_string());
            "put the request branch back where mortar left it, then mortar join"
        }
        _ => {
            let mut report = failure_report(VERB, &error.to_string());
            lines.append(&mut report.lines);
            report.lines = lines;
            return report;
        }
    };
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some(OWNER.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}
