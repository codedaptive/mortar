//! EntryKind: the type of a tree entry that is not a blob.

/// The type ls-tree names for an entry that is not a blob: `commit` for a gitlink (a submodule), `tree` for a folder,
/// and any other type kept as found (lossy UTF-8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// A gitlink: ls-tree's `commit`.
    Commit,
    /// A folder: ls-tree's `tree`.
    Tree,
    /// Any other type, as ls-tree named it.
    Other(String),
}
