//! FOLDER_TREE: the one place the name lives under which a resolve's final join-rebase op records folder_tree in
//! its values. join exports it for the cli.
//! No unit test: a declaration only; folder_tree_values_tests exercises it through folder_tree_values.

/// The values key under which a resolve's final join-rebase op records folder_tree.
pub const FOLDER_TREE: &str = "folder_tree";
