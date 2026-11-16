//! guard_state_report: foreign-hook, installed and not-installed, each with what it names.

use std::path::PathBuf;

use land::{GuardState, HookState, LandRefs};

use super::guard_state_report;
use crate::report::{ExitCode, OutValue, Report};

fn text(value: &str) -> OutValue {
    OutValue::Text(value.to_string())
}

fn refs(origin: &str, integration: &str, remote: &str) -> LandRefs {
    LandRefs {
        origin_branch: origin.to_string(),
        integration_branch: integration.to_string(),
        remote: remote.to_string(),
    }
}

fn config_refs() -> LandRefs {
    refs("main", "integration", "origin")
}

fn state(refs: Option<LandRefs>, reference_transaction: HookState, pre_push: HookState) -> GuardState {
    GuardState { refs, hooks_dir: PathBuf::from("/r/.git/hooks"), reference_transaction, pre_push }
}

#[test]
fn a_foreign_reference_transaction_is_refused_naming_its_path() {
    let report = guard_state_report(&state(None, HookState::Foreign, HookState::Missing), &config_refs());
    assert_eq!(
        report,
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Refused,
            outcome: "foreign-hook".to_string(),
            lines: vec![
                "/r/.git/hooks/reference-transaction is a hook Mortar did not write; Mortar never overwrites one"
                    .to_string()
            ],
            owner: Some("the repository's own hooks".to_string()),
            next: "move /r/.git/hooks/reference-transaction aside or fold it into your own hook by hand, then mortar \
                   guard install --actor <name>"
                .to_string(),
            fields: vec![
                ("hooks_dir".to_string(), text("/r/.git/hooks")),
                ("reference_transaction".to_string(), text("foreign")),
                ("pre_push".to_string(), text("missing")),
                ("origin_branch".to_string(), OutValue::Null),
                ("integration_branch".to_string(), OutValue::Null),
                ("remote".to_string(), OutValue::Null),
            ],
        }
    );
}

#[test]
fn both_hooks_foreign_name_both_and_the_next_names_reference_transaction() {
    let found = state(Some(config_refs()), HookState::Foreign, HookState::Foreign);
    let report = guard_state_report(&found, &config_refs());
    assert_eq!(report.outcome, "foreign-hook");
    assert_eq!(
        report.lines,
        vec![
            "/r/.git/hooks/reference-transaction is a hook Mortar did not write; Mortar never overwrites one"
                .to_string(),
            "/r/.git/hooks/pre-push is a hook Mortar did not write; Mortar never overwrites one".to_string(),
        ]
    );
    assert!(report.next.starts_with("move /r/.git/hooks/reference-transaction aside"), "{}", report.next);
}

#[test]
fn a_foreign_pre_push_beside_a_mortar_reference_transaction_is_refused() {
    let report = guard_state_report(&state(Some(config_refs()), HookState::Mortar, HookState::Foreign), &config_refs());
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(
        report.lines,
        vec!["/r/.git/hooks/pre-push is a hook Mortar did not write; Mortar never overwrites one".to_string()]
    );
}

#[test]
fn in_place_is_installed_naming_the_hooks_folder_and_branches() {
    let report = guard_state_report(&state(Some(config_refs()), HookState::Mortar, HookState::Mortar), &config_refs());
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "installed");
    assert_eq!(
        report.lines,
        vec![
            "the guard is installed in /r/.git/hooks".to_string(),
            "it guards refs/heads/integration and refs/heads/main, and pushes of them to origin".to_string(),
        ]
    );
    assert_eq!(report.owner, None);
    assert_eq!(report.next, "nothing to do: git now refuses moves of integration and main that Mortar did not make");
    assert_eq!(report.fields[3], ("origin_branch".to_string(), text("main")));
}

#[test]
fn an_untouched_repository_names_both_hooks_and_guard_json_missing() {
    assert_eq!(
        guard_state_report(&state(None, HookState::Missing, HookState::Missing), &config_refs()),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Done,
            outcome: "not-installed".to_string(),
            lines: vec![
                "the guard is not installed in /r/.git/hooks as the Mortar config asks".to_string(),
                "the reference-transaction hook is missing from /r/.git/hooks".to_string(),
                "the pre-push hook is missing from /r/.git/hooks".to_string(),
                "there is no guard.json, so the guard has no branches to check".to_string(),
            ],
            owner: None,
            next: "mortar guard install --actor <name>".to_string(),
            fields: vec![
                ("hooks_dir".to_string(), text("/r/.git/hooks")),
                ("reference_transaction".to_string(), text("missing")),
                ("pre_push".to_string(), text("missing")),
                ("origin_branch".to_string(), OutValue::Null),
                ("integration_branch".to_string(), OutValue::Null),
                ("remote".to_string(), OutValue::Null),
            ],
        }
    );
}

#[test]
fn one_missing_hook_is_named_alone() {
    let report = guard_state_report(&state(Some(config_refs()), HookState::Mortar, HookState::Missing), &config_refs());
    assert_eq!(report.outcome, "not-installed");
    assert_eq!(
        report.lines,
        vec![
            "the guard is not installed in /r/.git/hooks as the Mortar config asks".to_string(),
            "the pre-push hook is missing from /r/.git/hooks".to_string(),
        ]
    );
}

#[test]
fn a_guard_json_for_other_branches_names_both_sets() {
    let written = refs("main", "integration", "origin");
    let report = guard_state_report(
        &state(Some(written), HookState::Mortar, HookState::Mortar),
        &refs("release", "develop", "upstream"),
    );
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "not-installed");
    assert_eq!(
        report.lines,
        vec![
            "the guard is not installed in /r/.git/hooks as the Mortar config asks".to_string(),
            "guard.json guards refs/heads/integration and refs/heads/main and pushes to origin, but the Mortar config \
             names refs/heads/develop and refs/heads/release and pushes to upstream"
                .to_string(),
        ]
    );
    assert_eq!(report.next, "mortar guard install --actor <name>");
    assert_eq!(report.fields[3], ("origin_branch".to_string(), text("main")));
}
