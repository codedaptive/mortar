//! snapshot_report: the report of a `mortar snapshot` that ran (exit 0, written or not).

use snapshot::{FormatterResult, FormatterRun, SnapshotOutcome};

use crate::report::{ExitCode, OutValue, Report};

/// Done, next "nothing to do". Written: "snapshotted", naming the commit and its tree; not written: "unchanged",
/// naming the snapshot the ref already holds. Then one line per file the size cap left out, with its size and the
/// cap, then the formatter's run when one ran: its command, its paths and how it ended. Fields: commit, tree,
/// written and refused (the paths).
pub fn snapshot_report(outcome: &SnapshotOutcome) -> Report {
    let (word, first) = if outcome.written {
        ("snapshotted", format!("snapshotted commit {} (tree {})", outcome.commit, outcome.tree))
    } else {
        ("unchanged", format!("unchanged: the snapshot is commit {} (tree {})", outcome.commit, outcome.tree))
    };
    let mut lines = vec![first];
    lines.extend(outcome.refused.iter().map(|file| {
        format!(
            "not snapshotted: {} is {} bytes, over the size cap of {} bytes",
            file.path.display(),
            file.size,
            file.cap
        )
    }));
    lines.extend(outcome.formatted.as_ref().map(formatter_line));
    Report {
        verb: "snapshot".to_string(),
        exit: ExitCode::Done,
        outcome: word.to_string(),
        lines,
        owner: None,
        next: "nothing to do".to_string(),
        fields: vec![
            ("commit".to_string(), OutValue::Text(outcome.commit.to_string())),
            ("tree".to_string(), OutValue::Text(outcome.tree.to_string())),
            ("written".to_string(), OutValue::Bool(outcome.written)),
            (
                "refused".to_string(),
                OutValue::Texts(outcome.refused.iter().map(|file| file.path.display().to_string()).collect()),
            ),
        ],
    }
}

/// The formatter's command (program then its arguments), the paths it was given, and how it ended. A run that did
/// not end Formatted still let the snapshot go ahead, so its line says the files were snapshotted as they are.
fn formatter_line(run: &FormatterRun) -> String {
    let command = std::iter::once(run.program.display().to_string())
        .chain(run.args.iter().map(|arg| arg.to_string_lossy().into_owned()))
        .collect::<Vec<_>>()
        .join(" ");
    let paths = run.paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ");
    let kept = "the files were snapshotted as they are";
    let ended = match &run.result {
        FormatterResult::Formatted => "formatted".to_string(),
        FormatterResult::Failed { code: Some(code), .. } => format!("failed (exit {code}); {kept}"),
        FormatterResult::Failed { code: None, .. } => format!("failed (killed by a signal); {kept}"),
        FormatterResult::SpawnFailed { error } => format!("could not run: {error}; {kept}"),
        FormatterResult::TimedOut { limit, .. } => {
            format!("ran past its timeout of {} s and was stopped; {kept}", limit.as_secs())
        }
    };
    format!("formatter {command} on {paths}: {ended}")
}
