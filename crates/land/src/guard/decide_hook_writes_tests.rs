//! decide_hook_writes: each mix of Missing, Mortar and Foreign, with literal paths and the hook names the contract
//! gives (reference-transaction first, pre-push second).

use std::path::PathBuf;

use super::decide_hook_writes::decide_hook_writes;
use super::{GuardError, HookState};

fn hooks(first: HookState, second: HookState) -> [(PathBuf, HookState); 2] {
    [(PathBuf::from("/hooks/reference-transaction"), first), (PathBuf::from("/hooks/pre-push"), second)]
}

#[test]
fn both_missing_write_both_scripts() {
    assert_eq!(
        decide_hook_writes(&hooks(HookState::Missing, HookState::Missing)).unwrap(),
        vec![
            (PathBuf::from("/hooks/reference-transaction"), Some("reference-transaction")),
            (PathBuf::from("/hooks/pre-push"), Some("pre-push")),
        ]
    );
}

#[test]
fn both_mortar_only_set_modes() {
    assert_eq!(
        decide_hook_writes(&hooks(HookState::Mortar, HookState::Mortar)).unwrap(),
        vec![(PathBuf::from("/hooks/reference-transaction"), None), (PathBuf::from("/hooks/pre-push"), None)]
    );
}

#[test]
fn missing_and_mortar_mixed_write_only_the_missing_one() {
    assert_eq!(
        decide_hook_writes(&hooks(HookState::Mortar, HookState::Missing)).unwrap(),
        vec![
            (PathBuf::from("/hooks/reference-transaction"), None),
            (PathBuf::from("/hooks/pre-push"), Some("pre-push")),
        ]
    );
    assert_eq!(
        decide_hook_writes(&hooks(HookState::Missing, HookState::Mortar)).unwrap(),
        vec![
            (PathBuf::from("/hooks/reference-transaction"), Some("reference-transaction")),
            (PathBuf::from("/hooks/pre-push"), None),
        ]
    );
}

#[test]
fn foreign_first_is_refused_naming_it() {
    let result = decide_hook_writes(&hooks(HookState::Foreign, HookState::Missing));
    assert!(matches!(
        result,
        Err(GuardError::ForeignHook { ref path }) if path == &PathBuf::from("/hooks/reference-transaction")
    ));
}

#[test]
fn foreign_second_is_refused_naming_it() {
    let result = decide_hook_writes(&hooks(HookState::Mortar, HookState::Foreign));
    assert!(matches!(result, Err(GuardError::ForeignHook { ref path }) if path == &PathBuf::from("/hooks/pre-push")));
}

#[test]
fn both_foreign_name_the_first() {
    let result = decide_hook_writes(&hooks(HookState::Foreign, HookState::Foreign));
    assert!(matches!(
        result,
        Err(GuardError::ForeignHook { ref path }) if path == &PathBuf::from("/hooks/reference-transaction")
    ));
}
