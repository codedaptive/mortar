//! The refusal of an `mr guard` that found hooks Mortar did not write, in the cli's own words; guard_state_report
//! and guard_error_report each build on it, so status and install refuse a foreign hook alike.

use std::path::PathBuf;

use crate::report::{ExitCode, Report};

/// Exit 1 Refused, outcome "foreign-hook". Lines: one per path in `foreign`, "<path> is a hook Mortar did not
/// write; Mortar never overwrites one". Owner: "the repository's own hooks". Next: move the first path aside or fold
/// it into your own hook by hand, then `mr guard install --actor <name>`. Fields: none. `foreign` is never
/// empty (its callers pass at least the hook found foreign); were it empty, the next would name no path.
pub fn foreign_hook_report(foreign: &[PathBuf]) -> Report {
    let first = foreign.first().map(|path| path.display().to_string()).unwrap_or_default();
    Report {
        verb: "guard".to_string(),
        exit: ExitCode::Refused,
        outcome: "foreign-hook".to_string(),
        lines: foreign
            .iter()
            .map(|path| format!("{} is a hook Mortar did not write; Mortar never overwrites one", path.display()))
            .collect(),
        owner: Some("the repository's own hooks".to_string()),
        next: format!(
            "move {first} aside or fold it into your own hook by hand, then mr guard install --actor <name>"
        ),
        fields: Vec::new(),
    }
}
