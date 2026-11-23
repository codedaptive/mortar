//! The lines and --json fields of a report that say a pending folder move was finished or found already done: a
//! join rebase's, a joined partner request's or an undo's.
//! Shell API: save_part, resolve_report and save_report call it.

use super::{MoveCause, PendingMove};
use crate::report::OutValue;

/// Lines, when there was a move: for cause Rebase "the folder stands at the last rebase's tip: <old> -> <new>", for
/// cause Undo "the folder stands where the undo put the branch back: <old> -> <new>"; none without a move. Fields in
/// order: finished_move_from and finished_move_to, the move's old and new tips (Null when no move was pending).
pub fn finished_move_part(finished: Option<&PendingMove>) -> (Vec<String>, Vec<(String, OutValue)>) {
    let lines = finished
        .map(|moved| {
            let stands = match moved.cause {
                MoveCause::Rebase => "at the last rebase's tip",
                MoveCause::Undo => "where the undo put the branch back",
            };
            format!("the folder stands {stands}: {} -> {}", moved.old, moved.new)
        })
        .into_iter()
        .collect();
    let tip = |pick: fn(&PendingMove) -> String| finished.map_or(OutValue::Null, |moved| OutValue::Text(pick(moved)));
    let fields = vec![
        ("finished_move_from".to_string(), tip(|moved| moved.old.to_string())),
        ("finished_move_to".to_string(), tip(|moved| moved.new.to_string())),
    ];
    (lines, fields)
}
