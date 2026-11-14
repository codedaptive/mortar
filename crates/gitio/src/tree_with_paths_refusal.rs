//! TreeWithPathsRefusal: why Git::tree_with_paths_typed refused its paths before building anything.

use std::path::PathBuf;

/// One refusal of tree_with_paths_typed, carrying the path or paths it names, exactly as the caller gave them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeWithPathsRefusal {
    /// A path naming a folder in `from`: paths name file entries.
    FolderInFrom { path: PathBuf },
    /// A path `from` lacks that names a folder in `base`: removing it would drop only an entry of exactly that name,
    /// so the folder would stay.
    FolderInBase { path: PathBuf },
    /// Two paths where `folder` is a folder prefix of `inner` (`p` and `p/...`).
    Overlap { folder: PathBuf, inner: PathBuf },
    /// A path not written as git stores it: a leading or trailing '/', or an empty, '.' or '..' part.
    NotAsStored { path: PathBuf },
    /// An empty path, or one holding a NUL.
    Unusable { path: PathBuf },
}
