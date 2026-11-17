//! One entry of HEAD's tree as `git ls-tree -r -l` lists it. Data only: parse_ls_tree makes it and plan_checkout
//! reads it, and parse_ls_tree_tests.rs and plan_checkout_tests.rs cover how it is made and read.

/// The entry's mode as text ("100644", "100755", "120000", "160000"), its path from the top of the tree (it may
/// hold spaces), and its size in bytes, None for a submodule commit, which has none.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeFile {
    pub mode: String,
    pub path: String,
    pub size: Option<u64>,
}
