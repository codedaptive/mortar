//! tree_refusal: resolve's refusal for a failed build of the resolved tree, chosen by gitio's typed
//! TreeWithPathsError. tree_refusal_tests covers the rules that run no gitio code; the rest are RITs owed to
//! Nagatha (S-101, WO-join-3cr).

use crate::JoinRunError;
use gitio::{TreeWithPathsError, TreeWithPathsRefusal};
use std::path::PathBuf;

/// Maps a failure of tree_with_paths_typed over `files` (both callers pass the paths they asked for, and gitio names
/// each path as the caller gave it):
/// - FolderInFrom or FolderInBase is ConflictedPathIsFolder naming that path;
/// - Overlap is ConflictedPathRefused naming whichever of its two paths comes first in `files` (None if neither is
///   there);
/// - NotAsStored or Unusable is ConflictedPathRefused naming its path;
/// - Git with no exit code and no `args` is gitio's refusal of an answer of the wrong shape (an ls-tree listing, or
///   write-tree's id line), which has no typed variant: ConflictedPathRefused naming no path, its stderr as the reason;
/// - any other Git (a git run that failed, was killed or did not start) is Git, unchanged.
///
/// Each ConflictedPathRefused from a typed refusal keeps the refusal's Display text as its reason.
pub(crate) fn tree_refusal(error: TreeWithPathsError, files: &[PathBuf]) -> JoinRunError {
    let refusal = match error {
        TreeWithPathsError::Git(error) if error.code.is_none() && error.args.is_empty() => {
            return JoinRunError::ConflictedPathRefused { path: None, reason: error.stderr };
        }
        TreeWithPathsError::Git(error) => return JoinRunError::Git(error),
        TreeWithPathsError::Refused(refusal) => refusal,
    };
    let path = match &refusal {
        TreeWithPathsRefusal::FolderInFrom { path } | TreeWithPathsRefusal::FolderInBase { path } => {
            return JoinRunError::ConflictedPathIsFolder { path: path.clone() };
        }
        TreeWithPathsRefusal::Overlap { folder, inner } => {
            files.iter().find(|file| *file == folder || *file == inner).cloned()
        }
        TreeWithPathsRefusal::NotAsStored { path } | TreeWithPathsRefusal::Unusable { path } => Some(path.clone()),
    };
    JoinRunError::ConflictedPathRefused { path, reason: refusal.to_string() }
}
