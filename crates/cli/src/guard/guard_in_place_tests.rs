//! guard_in_place: true only for both hooks Mortar's and guard.json naming exactly the config's branches and remote.

use std::path::PathBuf;

use land::{GuardState, HookState, LandRefs};

use super::guard_in_place;

fn refs(origin: &str, integration: &str, remote: &str) -> LandRefs {
    LandRefs {
        origin_branch: origin.to_string(),
        integration_branch: integration.to_string(),
        remote: remote.to_string(),
    }
}

fn state(refs: Option<LandRefs>, reference_transaction: HookState, pre_push: HookState) -> GuardState {
    GuardState { refs, hooks_dir: PathBuf::from("/r/.git/hooks"), reference_transaction, pre_push }
}

fn config_refs() -> LandRefs {
    refs("main", "integration", "origin")
}

#[test]
fn both_hooks_mortar_and_the_same_refs_is_in_place() {
    assert!(guard_in_place(&state(Some(config_refs()), HookState::Mortar, HookState::Mortar), &config_refs()));
}

#[test]
fn any_hook_not_mortar_is_not_in_place() {
    for (reference_transaction, pre_push) in [
        (HookState::Missing, HookState::Mortar),
        (HookState::Foreign, HookState::Mortar),
        (HookState::Mortar, HookState::Missing),
        (HookState::Mortar, HookState::Foreign),
        (HookState::Missing, HookState::Missing),
        (HookState::Foreign, HookState::Foreign),
    ] {
        let found = state(Some(config_refs()), reference_transaction, pre_push);
        assert!(!guard_in_place(&found, &config_refs()), "{reference_transaction:?} {pre_push:?}");
    }
}

#[test]
fn no_guard_json_is_not_in_place() {
    assert!(!guard_in_place(&state(None, HookState::Mortar, HookState::Mortar), &config_refs()));
}

#[test]
fn guard_json_naming_another_branch_or_remote_is_not_in_place() {
    for other in [
        refs("release", "integration", "origin"),
        refs("main", "develop", "origin"),
        refs("main", "integration", "upstream"),
        refs("integration", "main", "origin"),
    ] {
        let found = state(Some(other.clone()), HookState::Mortar, HookState::Mortar);
        assert!(!guard_in_place(&found, &config_refs()), "{other:?}");
    }
}
