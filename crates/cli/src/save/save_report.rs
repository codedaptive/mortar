//! save_report: the report of a save that ran (S-71: exit 0, written or not), with the pending folder move it
//! finished first, if any (S-143).

use snapshot::{FormatterResult, FormatterRun, SaveOutcome};

use crate::join::{finished_move_part, PendingMove};
use crate::report::{ExitCode, OutValue, Report};

/// Done, next `mr join`. Lines: first finished_move_part's for `finished` (the move's line, none without a
/// move); then, written: "saved", naming the commit and its change ID; not written: "unchanged", naming HEAD. Then
/// one line per file the size cap refused, then, when the formatter did not end Formatted, how it ended (the files
/// were saved as they are). Fields: finished_move_part's (finished_move_from and finished_move_to, Null without a
/// move), then commit, written, change_id (Null when None), refused (the paths) and cap (the first refused file's
/// cap, Null when none was refused).
pub fn save_report(outcome: &SaveOutcome, formatted: Option<&FormatterRun>, finished: Option<&PendingMove>) -> Report {
    let change_id = outcome.change_id.as_ref().map(ToString::to_string);
    let (word, first) = if outcome.written {
        (
            "saved",
            format!(
                "saved commit {} on the request's branch, change ID {}",
                outcome.commit,
                change_id.as_deref().unwrap_or("none")
            ),
        )
    } else {
        (
            "unchanged",
            format!("nothing changed since the last save; no commit written (HEAD is {})", outcome.commit),
        )
    };
    let (mut lines, mut fields) = finished_move_part(finished);
    lines.push(first);
    lines.extend(outcome.refused.iter().map(|file| {
        format!(
            "not saved: {} is {} bytes, over the size cap of {} bytes",
            file.path.display(),
            file.size,
            file.cap
        )
    }));
    lines.extend(formatted.map(formatter_lines).unwrap_or_default());
    fields.extend([
        ("commit".to_string(), OutValue::Text(outcome.commit.to_string())),
        ("written".to_string(), OutValue::Bool(outcome.written)),
        ("change_id".to_string(), change_id.map_or(OutValue::Null, OutValue::Text)),
        (
            "refused".to_string(),
            OutValue::Texts(outcome.refused.iter().map(|file| file.path.display().to_string()).collect()),
        ),
        ("cap".to_string(), outcome.refused.first().map_or(OutValue::Null, |file| OutValue::Number(file.cap))),
    ]);
    Report {
        verb: "save".to_string(),
        exit: ExitCode::Done,
        outcome: word.to_string(),
        lines,
        owner: None,
        next: "mr join".to_string(),
        fields,
    }
}

/// How a formatter run that did not end Formatted ended; nothing for one that did.
fn formatter_lines(run: &FormatterRun) -> Vec<String> {
    let program = run.program.display();
    let kept = "the files were saved as they are";
    match &run.result {
        FormatterResult::Formatted => Vec::new(),
        FormatterResult::Failed { code, stderr } => {
            let ended = code.map_or("killed by a signal".to_string(), |code| format!("exit {code}"));
            let mut lines = vec![format!("the formatter {program} failed ({ended}); {kept}")];
            if !stderr.trim().is_empty() {
                lines.push(format!("formatter stderr: {}", stderr.trim()));
            }
            lines
        }
        FormatterResult::SpawnFailed { error } => {
            vec![format!("the formatter {program} could not run: {error}; {kept}")]
        }
        FormatterResult::TimedOut { limit, .. } => vec![format!(
            "the formatter {program} ran past its timeout of {} s and was stopped; {kept}",
            limit.as_secs()
        )],
    }
}
