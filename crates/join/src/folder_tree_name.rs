//! FOLDER_TREE: the one place the name lives under which a resolve's final join-rebase op records folder_tree in
//! its values, the name S-89(2) fixes. The cli's S-86 detector reads the same name from the op (WO-cli-3e).
//! No unit test: a declaration only (S-7); folder_tree_values_tests exercises it through folder_tree_values.

pub(crate) const FOLDER_TREE: &str = "folder_tree";
