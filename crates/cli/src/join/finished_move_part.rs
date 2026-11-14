//! The lines and --json fields of a report that say the last rebase's folder move was finished or found already
//! done (S-86).
//! Shell API: save_part and resolve_report call it.

use super::PendingMove;
use crate::report::OutValue;

/// Lines: "the folder stands at the last rebase's tip: <old> -> <new>" when there was a move, else none. Fields in
/// order: finished_move_from and finished_move_to, the move's old and new tips (Null when no move was pending).
pub fn finished_move_part(finished: Option<&PendingMove>) -> (Vec<String>, Vec<(String, OutValue)>) {
    let lines = finished
        .map(|moved| format!("the folder stands at the last rebase's tip: {} -> {}", moved.old, moved.new))
        .into_iter()
        .collect();
    let tip = |pick: fn(&PendingMove) -> String| finished.map_or(OutValue::Null, |moved| OutValue::Text(pick(moved)));
    let fields = vec![
        ("finished_move_from".to_string(), tip(|moved| moved.old.to_string())),
        ("finished_move_to".to_string(), tip(|moved| moved.new.to_string())),
    ];
    (lines, fields)
}
