//! BASE_BEFORE: the one place the name lives under which a join op records the request's base before the join in its
//! values. join exports it for the cli, as it does FOLDER_TREE.
//! No unit test of its own: a declaration only; base_values_tests and recorded_base_tests exercise it.

/// The values key under which a join-rebase or join-zone-request op records the base the request was replayed from.
pub const BASE_BEFORE: &str = "base_before";
