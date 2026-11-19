//! Whether an attempt that has a try branch but no folder is saved.

use gitio::Oid;

/// True only when the attempt's trysnap ref exists (`trysnap_tree` is its commit's tree) and that tree equals
/// `tip_tree`, the tree of the try branch's tip: then the snapshot holds everything the branch holds.
pub fn attempt_saved(trysnap_tree: Option<&Oid>, tip_tree: &Oid) -> bool {
    trysnap_tree == Some(tip_tree)
}
