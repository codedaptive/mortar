//! The report of the folder move after a resolve that folders::move_folder refused.
//! resolve_move_refusal_report_tests covers it (it calls move_refusal_report, which is pure, directly).

use folders::FolderError;
use gitio::Oid;

use crate::join::move_refusal_report;
use crate::report::{with_moved_first, ExitCode, Report};

const VERB: &str = "resolve";

/// `moved` (moved_before's line for what the snapshots before the move moved) is the first line of each report, then
/// `before` (what the resolve already did). Edits on paths the zone changed (MoveRefused) are exit 1
/// "refused", owned by the folder: the paths, then the snapshot ref and commit that keep them, since the snapshot
/// taken before join resolve holds them; next "put those files back to the folder's version, then mr join".
/// Anything else is move_refusal_report's report under the verb resolve.
pub fn resolve_move_refusal_report(
    error: &FolderError,
    snapshot_ref: &str,
    snapshot_commit: &Oid,
    before: Vec<String>,
    moved: Option<&str>,
) -> Report {
    let FolderError::MoveRefused { paths, .. } = error else {
        return Report { verb: VERB.to_string(), ..move_refusal_report(error, before, moved) };
    };
    let mut lines = with_moved_first(moved, before);
    lines.push("the folder could not move; edits on paths the zone changed stand in the way:".to_string());
    lines.extend(paths.iter().map(|path| format!("  {}", path.display())));
    lines.push(format!("they are kept in {snapshot_ref} at {snapshot_commit}"));
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some("this request's folder".to_string()),
        next: "put those files back to the folder's version, then mr join".to_string(),
        fields: Vec::new(),
    }
}
