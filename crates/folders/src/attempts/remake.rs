//! One orphan attempt to re-make, as remake_plan plans it.
//! A bare type; remake_plan_tests exercises it.

use std::path::PathBuf;

use gitio::Oid;

/// Attempt `k`, whose folder is `folder`, is re-made at a new commit holding `tree` (its snapshot's tree) with
/// `parent` (its snapshot's first parent) as its one parent; the folder's index is set to `tree`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remake {
    pub k: u32,
    pub folder: PathBuf,
    pub tree: Oid,
    pub parent: Oid,
}
