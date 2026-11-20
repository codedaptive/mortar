//! MergeTreeOutcome: what a merge of two commits into a tree gave. A declaration only; read_merge_tree_tests
//! exercises it.

use crate::Oid;
use std::path::PathBuf;

/// The merged tree, clean or holding conflicts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeTreeOutcome {
    /// The merge had no conflicts; this is the merged tree.
    Clean(Oid),
    /// The merge had conflicts: `tree` holds them with conflict markers, `files` names each conflicted path once,
    /// in git's order. A path in `files` may be absent from `tree`: a rename/rename conflict writes it under each
    /// new name instead.
    Conflicted { tree: Oid, files: Vec<PathBuf> },
}
