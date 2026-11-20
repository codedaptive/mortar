//! guard_fields: the six fields in order, each hook state's word, and nulls without a guard.json.

use std::path::PathBuf;

use land::{GuardState, HookState, LandRefs};

use super::guard_fields;
use crate::report::OutValue;

fn text(value: &str) -> OutValue {
    OutValue::Text(value.to_string())
}

fn field(name: &str, value: OutValue) -> (String, OutValue) {
    (name.to_string(), value)
}

#[test]
fn with_a_guard_json_its_three_names_follow_the_hooks() {
    let state = GuardState {
        refs: Some(LandRefs {
            origin_branch: "release".to_string(),
            integration_branch: "develop".to_string(),
            remote: "upstream".to_string(),
        }),
        hooks_dir: PathBuf::from("/r/myhooks"),
        reference_transaction: HookState::Mortar,
        pre_push: HookState::Foreign,
    };
    assert_eq!(
        guard_fields(&state),
        vec![
            field("hooks_dir", text("/r/myhooks")),
            field("reference_transaction", text("mortar")),
            field("pre_push", text("foreign")),
            field("origin_branch", text("release")),
            field("integration_branch", text("develop")),
            field("remote", text("upstream")),
        ]
    );
}

#[test]
fn without_a_guard_json_the_three_names_are_null() {
    let state = GuardState {
        refs: None,
        hooks_dir: PathBuf::from("/r/.git/hooks"),
        reference_transaction: HookState::Missing,
        pre_push: HookState::Mortar,
    };
    assert_eq!(
        guard_fields(&state),
        vec![
            field("hooks_dir", text("/r/.git/hooks")),
            field("reference_transaction", text("missing")),
            field("pre_push", text("mortar")),
            field("origin_branch", OutValue::Null),
            field("integration_branch", OutValue::Null),
            field("remote", OutValue::Null),
        ]
    );
}
