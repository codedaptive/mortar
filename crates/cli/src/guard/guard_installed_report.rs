//! The report of a `mortar guard install` that left the guard in place, and of a `mortar guard status` that found it
//! in place (guard_state_report).

use land::GuardState;

use super::guard_fields;
use crate::report::{ExitCode, Report};

/// Exit 0 Done, outcome "already-installed" when `already` (the guard was in place before, nothing was written),
/// else "installed". Owner: none. Lines: the guard is installed (or already installed, nothing written) in
/// `state.hooks_dir`; then that it guards refs/heads/<integration> and refs/heads/<origin> and pushes of them to
/// <remote>, as `state.refs` names them. Next: "nothing to do: git now refuses moves of <integration> and <origin>
/// that Mortar did not make". Fields: guard_fields(state). With no guard.json in `state` (install wrote one, so only
/// when it vanished before it was read back) the second line says so and next is `mortar guard install --actor
/// <name>`.
pub fn guard_installed_report(already: bool, state: &GuardState) -> Report {
    let hooks_dir = state.hooks_dir.display();
    let (outcome, first) = if already {
        ("already-installed", format!("the guard is already installed in {hooks_dir}; nothing was written"))
    } else {
        ("installed", format!("the guard is installed in {hooks_dir}"))
    };
    let (second, next) = match &state.refs {
        Some(refs) => {
            let (integration, origin) = (&refs.integration_branch, &refs.origin_branch);
            (
                format!(
                    "it guards refs/heads/{integration} and refs/heads/{origin}, and pushes of them to {}",
                    refs.remote
                ),
                format!("nothing to do: git now refuses moves of {integration} and {origin} that Mortar did not make"),
            )
        }
        None => (
            "guard.json names no branches to guard".to_string(),
            "mortar guard install --actor <name>".to_string(),
        ),
    };
    Report {
        verb: "guard".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: vec![first, second],
        owner: None,
        next,
        fields: guard_fields(state),
    }
}
