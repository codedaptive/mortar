//! JOIN_STATE: the one place the name lives under which a resolve's final join-rebase op records, beside
//! folder_tree, the state mortar-join.json held before that final run (S-142(1)). join exports it,
//! as it does FOLDER_TREE (S-106).
//! No unit test of its own: a declaration only (S-7); join_state_value_tests exercises it through join_state_value.

/// The values key under which a resolve's final join-rebase op records the paused state it resolved from.
pub const JOIN_STATE: &str = "join_state";
