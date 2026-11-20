//! The report of `mr init`.

use std::path::Path;

use super::InitDecision;
use crate::report::{ExitCode, OutValue, Report};

/// The step after init: making a zone's folder.
const NEXT_DONE: &str = "mr folder zone <zone> <path>";

/// Write and Replace: done, "written" or "replaced", naming the path and for Replace the keys changed.
/// AlreadyDone: done, "already-done", nothing changed. Refused: exit 1, naming the keys that differ, owned by the
/// existing config. Every arm carries the path as the --json field config_path.
pub fn init_report(decision: &InitDecision, path: &Path) -> Report {
    let shown = path.display();
    let (exit, outcome, lines, owner, next) = match decision {
        InitDecision::Write => (ExitCode::Done, "written", vec![format!("wrote the Mortar config {shown}")], None, NEXT_DONE),
        InitDecision::Replace { keys } => (
            ExitCode::Done,
            "replaced",
            vec![format!("replaced the Mortar config {shown}; changed: {}", keys.join(", "))],
            None,
            NEXT_DONE,
        ),
        InitDecision::AlreadyDone => (
            ExitCode::Done,
            "already-done",
            vec!["nothing changed".to_string(), format!("the Mortar config {shown} already holds these values")],
            None,
            NEXT_DONE,
        ),
        InitDecision::Refused { keys } => (
            ExitCode::Refused,
            "refused",
            vec![
                format!("the Mortar config {shown} holds other values for: {}", keys.join(", ")),
                "nothing changed".to_string(),
            ],
            Some(format!("the existing config at {shown}")),
            "mr init with the same values, or add --replace",
        ),
    };
    Report {
        verb: "init".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines,
        owner,
        next: next.to_string(),
        fields: vec![("config_path".to_string(), OutValue::Text(path.display().to_string()))],
    }
}
