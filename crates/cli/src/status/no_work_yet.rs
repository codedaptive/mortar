//! no_work_yet: whether a folder holds no work of its own yet (S-145).

use gitio::Oid;

use super::is_saved;

/// True when the branch tip (`branch_tip`) is still the folder record's `base` and the folder is saved (is_saved: its
/// tree, `folder_tree`, equals the tip's tree, `tip_tree`). Commits of its own, or files that differ from the tip,
/// are work (S-145).
pub fn no_work_yet(base: &Oid, branch_tip: &Oid, folder_tree: &Oid, tip_tree: &Oid) -> bool {
    branch_tip == base && is_saved(folder_tree, tip_tree)
}
