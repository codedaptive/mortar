//! The lines and --json fields of a join report that say what was saved and the last rebase's folder move, finished
//! or found already done.

use snapshot::SaveOutcome;

use super::PendingMove;
use crate::report::OutValue;

/// Lines: the last rebase's folder move, finished or found already done (S-86), when there is one, then the
/// saved commit and its change ID (or "no unsaved work"), then each file the size cap left out.
/// Fields in order: finished_move_from, finished_move_to (Null when no move was pending), saved_commit,
/// saved_written, change_id (Null when nothing was written), refused (the paths).
pub fn save_part(save: &SaveOutcome, finished: Option<&PendingMove>) -> (Vec<String>, Vec<(String, OutValue)>) {
    let mut lines = Vec::new();
    if let Some(moved) = finished {
        lines.push(format!("the folder stands at the last rebase's tip: {} -> {}", moved.old, moved.new));
    }
    lines.push(match (save.written, &save.change_id) {
        (true, Some(id)) => format!("work saved at join as {} (change {id})", save.commit),
        (true, None) => format!("work saved at join as {}", save.commit),
        (false, _) => "no unsaved work".to_string(),
    });
    for file in &save.refused {
        lines.push(format!(
            "not saved, over the size cap: {} ({} bytes, cap {})",
            file.path.display(),
            file.size,
            file.cap
        ));
    }
    let tip = |pick: fn(&PendingMove) -> String| finished.map_or(OutValue::Null, |moved| OutValue::Text(pick(moved)));
    let fields = vec![
        ("finished_move_from".to_string(), tip(|moved| moved.old.to_string())),
        ("finished_move_to".to_string(), tip(|moved| moved.new.to_string())),
        ("saved_commit".to_string(), OutValue::Text(save.commit.to_string())),
        ("saved_written".to_string(), OutValue::Bool(save.written)),
        ("change_id".to_string(), save.change_id.as_ref().map_or(OutValue::Null, |id| OutValue::Text(id.to_string()))),
        (
            "refused".to_string(),
            OutValue::Texts(save.refused.iter().map(|file| file.path.display().to_string()).collect()),
        ),
    ];
    (lines, fields)
}
