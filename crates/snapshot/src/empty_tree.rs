//! empty_tree: the id of git's empty tree, in the same format as a given id.

use gitio::{GitError, Oid};

const SHA1_EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const SHA256_EMPTY_TREE: &str = "6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321";

/// The sha1 empty tree when `like` is 40 hex characters, else the sha256 one. Diffing from it lists every path in
/// a tree; git diff accepts the empty tree id without the object being stored (checked on sha1 and sha256 repos).
/// A Result because only Oid::parse makes an Oid outside gitio; both ids are valid, so it does not fail.
pub(crate) fn empty_tree(like: &Oid) -> Result<Oid, GitError> {
    Oid::parse(if like.as_str().len() == 40 { SHA1_EMPTY_TREE } else { SHA256_EMPTY_TREE })
}
