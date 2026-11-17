//! is_resolve_step: a join-rebase op carrying a "join_state" value is a resolve's final join step; a join-rebase
//! with folder_tree only, another kind, or no values is not.

use oplog::{OpPhase, OpRecord, ValuedRecord};

use super::is_resolve_step;

fn record(kind: &str, values: &[(&str, &str)]) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: "op-5".to_string(),
            key: format!("{kind}:op-5"),
            kind: kind.to_string(),
            actor: "agent-a".to_string(),
            reason: "test".to_string(),
            at: 1_700_000_000,
            phase: OpPhase::Done,
            moves: Vec::new(),
            result: None,
        },
        values: values.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect(),
    }
}

const TREE: &str = "4444444444444444444444444444444444444444";

#[test]
fn a_join_rebase_with_join_state_is_a_resolve_step() {
    assert!(is_resolve_step(&record("join-rebase", &[("folder_tree", TREE), ("join_state", "{\"paused\":true}")])));
}

#[test]
fn any_join_state_text_counts() {
    for text in ["", "not json", "{}"] {
        assert!(is_resolve_step(&record("join-rebase", &[("join_state", text)])), "{text:?}");
    }
}

#[test]
fn a_join_rebase_with_folder_tree_only_is_not() {
    assert!(!is_resolve_step(&record("join-rebase", &[("folder_tree", TREE)])));
}

#[test]
fn another_kind_with_join_state_is_not() {
    for kind in ["save", "undo", "join-ff", "folder-move"] {
        assert!(!is_resolve_step(&record(kind, &[("join_state", "{}")])), "{kind}");
    }
}

#[test]
fn a_join_rebase_with_no_values_is_not() {
    assert!(!is_resolve_step(&record("join-rebase", &[])));
}

#[test]
fn a_value_whose_text_is_join_state_under_another_name_is_not() {
    assert!(!is_resolve_step(&record("join-rebase", &[("other", "join_state")])));
}
