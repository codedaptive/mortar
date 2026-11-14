//! same_squash: the repeat decision, whether a squash would rewrite the commit already at its ref.

use gitio::{CommitInfo, Oid};

/// True when `prior` has exactly this tree, this one parent and this message, byte for byte: the squash then
/// writes nothing and answers with the prior commit, so a repeated squash is safe.
pub(crate) fn same_squash(prior: &CommitInfo, tree: &Oid, parent: &Oid, message: &str) -> bool {
    &prior.tree == tree && prior.parents.as_slice() == std::slice::from_ref(parent) && prior.message == message
}
