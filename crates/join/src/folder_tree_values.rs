//! folder_tree_values: the folder_tree value, one of the values a resolve's final join-rebase op records (S-89).

use crate::folder_tree_name::FOLDER_TREE;
use gitio::Oid;

/// Answers the op's values for `folder_tree`: one pair, FOLDER_TREE and the tree's hex. With no folder_tree it
/// refuses with the reason: an op without the value would carry none, Rebased would answer None and the caller
/// would move the folder from `from`, which git refuses after a resolve (S-87); the state is kept and no ref is
/// moved.
pub(crate) fn folder_tree_values(folder_tree: Option<&Oid>) -> Result<Vec<(String, String)>, String> {
    match folder_tree {
        Some(tree) => Ok(vec![(FOLDER_TREE.to_string(), tree.as_str().to_string())]),
        None => Err("the state's rebased_to is set but its folder_tree is null, so the final op could not record \
                     folder_tree (S-89); the state is kept and no ref moved"
            .to_string()),
    }
}
