//! CommitPathsError: why commit_paths wrote no commit, or wrote one and could not set the folder's index. A
//! declaration only: its Display is commit_paths_error_display, tested per variant in
//! commit_paths_error_display_tests.rs; outside_path_tests gives Absolute and ParentPart. The integration tests in
//! tests/snapshot/commit_paths_cases.rs get Absolute (commit_paths_refuses_absolute), ParentPart
//! (commit_paths_refuses_parent_part), ThroughSymlink (commit_paths_refuses_symlink_ancestor) and NotAdded
//! (commit_paths_refuses_ignored_named_file_without_a_commit). Ancestor, Paths, Git and IndexNotSet have no
//! integration test.

use gitio::Oid;
use std::path::PathBuf;

/// Derives Debug only, since std::io::Error is neither Clone nor Eq. Every variant but IndexNotSet means no commit
/// was written and the index was not touched; the path checks (Absolute to Ancestor) come before any git run, so
/// they write nothing at all.
#[derive(Debug)]
pub enum CommitPathsError {
    /// The path has a root; paths are named relative to the folder.
    Absolute { path: PathBuf },
    /// The path has a `..` part, which could leave the folder.
    ParentPart { path: PathBuf },
    /// A folder on the way to the path is a symlink, which could leave the folder; `link` is that folder, relative
    /// to the folder.
    ThroughSymlink { path: PathBuf, link: PathBuf },
    /// A folder on the way to the path could not be looked at (an error other than not found).
    Ancestor { path: PathBuf, error: std::io::Error },
    /// The path is on disk (any kind) but absent from the folder's tree as git built it: git ignores it.
    NotAdded { path: PathBuf },
    /// gitio's tree_with_paths_typed refused the paths.
    Paths(gitio::TreeWithPathsRefusal),
    /// A git run failed before the commit was written.
    Git(gitio::GitError),
    /// The commit was written but the folder's index could not be set for the named paths. No ref moved and the
    /// files are untouched. The index is the old one, unless gitio's set_index failed at its sparse-checkout
    /// reapply, after read-tree had already set it: then it holds the merged tree without its skip-worktree bits.
    IndexNotSet { commit: Oid, error: gitio::IndexTreeError },
}

impl std::error::Error for CommitPathsError {}
