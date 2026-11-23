//! Display for CommitPathsError: one plain sentence per variant, saying what was written; wrapped errors show
//! through their own Display. commit_paths_error_display_tests.rs checks every variant's text.

use crate::CommitPathsError;
use std::fmt;

impl fmt::Display for CommitPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Paths show in their Debug form, so a name that is not UTF-8 or holds a newline shows escaped.
        match self {
            CommitPathsError::Absolute { path } => write!(
                f,
                "the path {path:?} is absolute; paths are named relative to the folder; nothing was written"
            ),
            CommitPathsError::ParentPart { path } => write!(
                f,
                "the path {path:?} has a '..' part, which could leave the folder; nothing was written"
            ),
            CommitPathsError::ThroughSymlink { path, link } => write!(
                f,
                "the path {path:?} goes through the symlink {link:?}, which could leave the folder; nothing was written"
            ),
            CommitPathsError::Ancestor { path, error } => write!(
                f,
                "a folder on the way to the path {path:?} could not be looked at: {error}; nothing was written"
            ),
            CommitPathsError::NotAdded { path } => write!(
                f,
                "the path {path:?} is on disk but git ignores it, so it is not in the folder's tree; \
                 no commit was written"
            ),
            CommitPathsError::Paths(refusal) => write!(f, "the paths are refused: {refusal}; no commit was written"),
            CommitPathsError::Git(error) => write!(f, "a git run failed: {error}; no commit was written"),
            CommitPathsError::IndexNotSet { commit, error } => write!(
                f,
                "commit {commit} was written but the folder's index could not be set for its paths: {error}; \
                 no ref moved, the files are untouched and the index is the old one, or the merged tree without \
                 its skip-worktree bits if the sparse-checkout reapply failed"
            ),
        }
    }
}
