//! PathKind: what a path names in a tree.

/// The kind of entry a path names in a tree, as ls-tree types it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PathKind {
    /// A blob: a regular or executable file, or a symlink (mode 120000). A symlink is File because git stores it as a
    /// blob holding its target; path_kind does not follow it.
    File,
    /// A tree: a folder.
    Folder,
    /// A commit entry: a gitlink (a submodule).
    Gitlink,
}
