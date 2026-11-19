//! Git::merge_tree: merge two commits into a tree without touching any folder. It only runs git, so it has no unit
//! tests; tests/typed/merge.rs covers it: Clean with no folder touched, Conflicted with the file names, an error
//! for an unknown commit, and the attribute source's merge drivers.

use crate::merge_tree_request::merge_tree_request;
use crate::read_merge_tree::read_merge_tree;
use crate::{Git, GitError, MergeTreeOutcome, Oid};

impl Git {
    /// Merges `theirs` into `ours` (from `merge_base` when given, else git's own merge base), reading merge
    /// attributes from `attr_source`. Writes the merged tree object either way and returns it, with the conflicted
    /// files when there are conflicts. A conflicted path may be absent from the written tree: a rename/rename
    /// conflict writes it under each new name instead.
    pub fn merge_tree(
        &self,
        attr_source: &Oid,
        merge_base: Option<&Oid>,
        ours: &Oid,
        theirs: &Oid,
    ) -> Result<MergeTreeOutcome, GitError> {
        let (args, opts) = merge_tree_request(attr_source, merge_base, ours, theirs);
        read_merge_tree(&self.run(&args, &opts)?)
    }
}
