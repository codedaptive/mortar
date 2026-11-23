//! The lines and --json fields of a join report that say what was saved and the pending folder move (the last
//! rebase's, a joined partner request's or an undo's), finished or found already done.

use snapshot::SaveOutcome;

use super::{finished_move_part, PendingMove};
use crate::report::OutValue;

/// Lines: finished_move_part's (the pending folder move, finished or found already done), then the
/// saved commit and its change ID (or "no unsaved work"), then each file the size cap left out.
/// Fields in order: finished_move_part's (finished_move_from, finished_move_to), saved_commit, saved_written,
/// change_id (Null when nothing was written), refused (the paths).
pub fn save_part(save: &SaveOutcome, finished: Option<&PendingMove>) -> (Vec<String>, Vec<(String, OutValue)>) {
    let (mut lines, mut fields) = finished_move_part(finished);
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
    fields.extend([
        ("saved_commit".to_string(), OutValue::Text(save.commit.to_string())),
        ("saved_written".to_string(), OutValue::Bool(save.written)),
        ("change_id".to_string(), save.change_id.as_ref().map_or(OutValue::Null, |id| OutValue::Text(id.to_string()))),
        (
            "refused".to_string(),
            OutValue::Texts(save.refused.iter().map(|file| file.path.display().to_string()).collect()),
        ),
    ]);
    (lines, fields)
}
