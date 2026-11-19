//! JOIN_STATE: the one place the name lives under which a resolve's final join-rebase op records, beside
//! folder_tree, the state mortar-join.json held before that final run. join exports it, as it does
//! FOLDER_TREE.
//! No unit test of its own: a declaration only; join_state_value_tests exercises it through join_state_value.

/// The values key under which a resolve's final join-rebase op records the paused state it resolved from.
pub const JOIN_STATE: &str = "join_state";
