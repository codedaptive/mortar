//! The report of a `mortar sweep` that finished.

use folders::SweepBuildOutcome;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0. Outcome "swept" when a ref was deleted or a build dir removed, naming each, else "nothing-to-sweep" with
/// the line "nothing to sweep". With `build` (--idle-days given), the bytes its build dirs freed; without, a line
/// that the build dirs were not swept. Fields: deleted, removed, freed_bytes (null without --idle-days).
pub fn sweep_report(deleted: &[String], build: Option<&SweepBuildOutcome>) -> Report {
    let removed: Vec<String> =
        build.map_or_else(Vec::new, |b| b.removed.iter().map(|path| path.display().to_string()).collect());
    let swept = !deleted.is_empty() || !removed.is_empty();
    let mut lines: Vec<String> = deleted.iter().map(|refname| format!("deleted {refname}")).collect();
    lines.extend(removed.iter().map(|path| format!("removed build dir {path}")));
    if !swept {
        lines.push("nothing to sweep".to_string());
    }
    lines.push(match build {
        Some(b) => format!("freed {} bytes of build dirs", b.freed_bytes),
        None => "build dirs not swept: no --idle-days".to_string(),
    });
    Report {
        verb: "sweep".to_string(),
        exit: ExitCode::Done,
        outcome: if swept { "swept" } else { "nothing-to-sweep" }.to_string(),
        lines,
        owner: None,
        next: "nothing to do".to_string(),
        fields: vec![
            ("deleted".to_string(), OutValue::Texts(deleted.to_vec())),
            ("removed".to_string(), OutValue::Texts(removed)),
            ("freed_bytes".to_string(), build.map_or(OutValue::Null, |b| OutValue::Number(b.freed_bytes))),
        ],
    }
}
