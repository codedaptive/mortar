//! The report of a folder move that folders::move_folder refused during `mr join`. folders' own Display text is
//! never printed: it carries next steps that are not cli commands; the facts are folder_error_why's.

use folders::FolderError;

use crate::folder::{folder_error_why, folder_refusal};
use crate::lock::lock_report;
use crate::report::{failure_report, with_moved_first, ExitCode, Report};

const VERB: &str = "join";
const OWNER: &str = "this request's folder";

/// `moved` (moved_before's line for the refs the save and the snapshots before the move moved: run_join passes the
/// save's branch movement with them, finish_pending_move the snapshot's alone) is the first line of each report, then
/// `before` (what the join already did). Files in the way (MoveRefused) are exit 1 "refused": they
/// are listed, and once they are out of the way the next join, resolve, save or undo finishes the move first, since
/// the request branch already stands at the new tip. A branch at neither tip (BranchElsewhere) is exit 1
/// "refused" with folder_error_why's line. Any other variant folder_refusal refuses is exit 1 "refused" with
/// folder_error_why's lines and folder_refusal's next. The rest (Lock, Git, Oplog, DiskProbe, and Io, which
/// folder_refusal makes a failure) are fallback_report's report with `moved` and `before` ahead of its lines. Every
/// refusal is owned by this request's folder.
pub fn move_refusal_report(error: &FolderError, before: Vec<String>, moved: Option<&str>) -> Report {
    let mut lines = with_moved_first(moved, before);
    let next = match error {
        FolderError::MoveRefused { paths, .. } => {
            lines.push("the folder could not move; these files stand in the way:".to_string());
            lines.extend(paths.iter().map(|path| format!("  {}", path.display())));
            lines.push(
                "the request branch already stands at the new tip; the next mr join, mr resolve, mr save or mr undo \
                 finishes the move first"
                    .to_string(),
            );
            "move those files out of the way, then mr join".to_string()
        }
        FolderError::BranchElsewhere { .. } => {
            lines.extend(folder_error_why(error));
            "put the request branch back where mortar left it, then mr join".to_string()
        }
        other => match folder_refusal(other) {
            Some((ExitCode::Refused, next)) => {
                lines.extend(folder_error_why(other));
                next
            }
            _ => {
                let mut report = fallback_report(other);
                lines.append(&mut report.lines);
                report.lines = lines;
                return report;
            }
        },
    };
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some(OWNER.to_string()),
        next,
        fields: Vec::new(),
    }
}

/// Lock is lock_report("join") with each line's "; nothing ran" scoped to "; the folder move did not run": the
/// snapshot before the move may already have written. Git and Oplog are failure_report with the inner error's own text (gitio's and
/// oplog's), DiskProbe with its path and reason. Every other variant (Io, the one folder_refusal makes a failure)
/// is failure_report with folder_error_why's lines joined by "; ".
fn fallback_report(error: &FolderError) -> Report {
    match error {
        FolderError::Lock(lock) => {
            let mut report = lock_report(VERB, lock);
            report.lines = report.lines.into_iter().map(scope_to_the_move).collect();
            report
        }
        FolderError::Git(git) => failure_report(VERB, &git.to_string()),
        FolderError::Oplog(oplog) => failure_report(VERB, &oplog.to_string()),
        FolderError::DiskProbe { path, reason } => {
            failure_report(VERB, &format!("the free disk at {} could not be read: {reason}", path.display()))
        }
        other => failure_report(VERB, &folder_error_why(other).join("; ")),
    }
}

/// lock_report's line with its closing "; nothing ran" made "; the folder move did not run"; any other line as is.
fn scope_to_the_move(line: String) -> String {
    match line.strip_suffix("; nothing ran") {
        Some(head) => format!("{head}; the folder move did not run"),
        None => line,
    }
}
