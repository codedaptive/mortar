//! FOLDER_TREE: the one place the name lives under which a resolve's final join-rebase op records folder_tree in
//! its values, the name S-89(2) fixes. join exports it for the cli (S-106).
//! No unit test: a declaration only (S-7); folder_tree_values_tests exercises it through folder_tree_values.

/// The values key under which a resolve's final join-rebase op records folder_tree.
pub const FOLDER_TREE: &str = "folder_tree";
