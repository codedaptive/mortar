//! The --json fields every `mortar guard` report of a GuardState carries (guard_state_report and
//! guard_installed_report), so both name the hooks folder, each hook and guard.json's branches alike.

use land::{GuardState, HookState, LandRefs};

use crate::report::OutValue;

/// In order: hooks_dir; reference_transaction and pre_push, each "missing", "mortar" or "foreign"; then
/// origin_branch, integration_branch and remote from guard.json, each null when there is no guard.json.
pub fn guard_fields(state: &GuardState) -> Vec<(String, OutValue)> {
    let from_config = |name: fn(&LandRefs) -> &String| {
        state.refs.as_ref().map_or(OutValue::Null, |refs| OutValue::Text(name(refs).clone()))
    };
    vec![
        ("hooks_dir".to_string(), OutValue::Text(state.hooks_dir.display().to_string())),
        ("reference_transaction".to_string(), hook_text(state.reference_transaction)),
        ("pre_push".to_string(), hook_text(state.pre_push)),
        ("origin_branch".to_string(), from_config(|refs| &refs.origin_branch)),
        ("integration_branch".to_string(), from_config(|refs| &refs.integration_branch)),
        ("remote".to_string(), from_config(|refs| &refs.remote)),
    ]
}

fn hook_text(hook: HookState) -> OutValue {
    let text = match hook {
        HookState::Missing => "missing",
        HookState::Mortar => "mortar",
        HookState::Foreign => "foreign",
    };
    OutValue::Text(text.to_string())
}
