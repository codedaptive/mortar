//! What the save and snapshot before a join, a resolve or a save moved, as the report's line names it.
//! Shell API: join_report, resolve_report, run_join, run_resolve, run_save and finish_pending_move call it;
//! moved_before_tests covers it.

/// None when neither ref moved; otherwise the refs that moved, `request_ref` (moved by the save) before the snapshot
/// ref (moved by the snapshot step), followed by "moved as the work was saved first".
pub(crate) fn moved_before(branch_moved: bool, snapshot_moved: bool, request_ref: &str) -> Option<String> {
    let refs = match (branch_moved, snapshot_moved) {
        (false, false) => return None,
        (true, false) => request_ref.to_string(),
        (false, true) => "the snapshot ref".to_string(),
        (true, true) => format!("{request_ref} and the snapshot ref"),
    };
    Some(format!("{refs} moved as the work was saved first"))
}
