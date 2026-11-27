//! The report of `mr guard status`: the guard as guard_status found it, against the branches the config names.

use std::path::PathBuf;

use land::{GuardState, HookState, LandRefs};

use super::{foreign_hook_report, guard_fields, guard_in_place, guard_installed_report};
use crate::report::{ExitCode, Report};

/// The hooks the guard installs, in the order land reads and refuses them.
const HOOKS: [&str; 2] = ["reference-transaction", "pre-push"];

/// Fields in every case: guard_fields(state).
/// - Any hook Foreign: exit 3 Refused, outcome "foreign-hook", foreign_hook_report's lines (one per foreign hook,
///   naming `<hooks_dir>/<name>`), owner "the repository's own hooks", next moving the first aside, then
///   `mr guard install --actor <name>`.
/// - Else in place (guard_in_place against `refs`): guard_installed_report's exit 0 Done, outcome "installed", lines
///   (installed in hooks_dir, guarding refs/heads/<integration> and refs/heads/<origin> and pushes to <remote>), no
///   owner, next "nothing to do: git now refuses moves of <integration> and <origin> that Mortar did not make".
/// - Else exit 0 Done, outcome "not-installed", no owner. Lines: not installed in hooks_dir as the config asks, then
///   each missing hook by name, a missing guard.json, or a guard.json naming other branches or remote than `refs`
///   (both named). Next: `mr guard install --actor <name>`.
pub fn guard_state_report(state: &GuardState, refs: &LandRefs) -> Report {
    let found = [state.reference_transaction, state.pre_push];
    let foreign: Vec<PathBuf> = HOOKS
        .iter()
        .zip(found)
        .filter(|(_, hook)| *hook == HookState::Foreign)
        .map(|(name, _)| state.hooks_dir.join(name))
        .collect();
    if !foreign.is_empty() {
        return Report { fields: guard_fields(state), ..foreign_hook_report(&foreign) };
    }
    if guard_in_place(state, refs) {
        return guard_installed_report(false, state);
    }
    let hooks_dir = state.hooks_dir.display();
    let mut lines = vec![format!("the guard is not installed in {hooks_dir} as the Mortar config asks")];
    for (name, _) in HOOKS.iter().zip(found).filter(|(_, hook)| *hook == HookState::Missing) {
        lines.push(format!("the {name} hook is missing from {hooks_dir}"));
    }
    match &state.refs {
        None => lines.push("there is no guard.json, so the guard has no branches to check".to_string()),
        Some(written) if written != refs => lines.push(format!(
            "guard.json guards {} and pushes to {}, but the Mortar config names {} and pushes to {}",
            branches(written),
            written.remote,
            branches(refs),
            refs.remote
        )),
        Some(_) => {}
    }
    Report {
        verb: "guard".to_string(),
        exit: ExitCode::Done,
        outcome: "not-installed".to_string(),
        lines,
        owner: None,
        next: "mr guard install --actor <name>".to_string(),
        fields: guard_fields(state),
    }
}

/// "refs/heads/<integration> and refs/heads/<origin>".
fn branches(refs: &LandRefs) -> String {
    format!("refs/heads/{} and refs/heads/{}", refs.integration_branch, refs.origin_branch)
}
