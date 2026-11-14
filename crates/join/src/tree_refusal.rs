//! tree_refusal: resolve's refusal for a failed build of the resolved tree. tree_refusal_tests covers each rule.
//! gitio's GitError has no variants: a refusal has no exit code, no `args` and its reason in `stderr`, naming each
//! path in its `{:?}` form (gitio's tree_with_paths_list_request, read_path_entries, usable_path). A git run killed
//! by a signal, or one that did not start, also has no exit code, but it keeps its `args`.

use crate::JoinRunError;
use gitio::GitError;
use std::path::PathBuf;

/// An error with an exit code or with `args` (a git run that failed, was killed or did not start) is Git, unchanged.
/// A refusal is matched against gitio's whole message shapes, each path in its `{:?}` form, so a path whose form
/// holds another's never names the other:
/// - exactly `not a file entry: <path> is a folder` is ConflictedPathIsFolder naming that path;
/// - ending in `: <a> and <b>` for two of `files` (the overlap refusal) is ConflictedPathRefused naming whichever of
///   the two comes first in `files`;
/// - ending in `: <path>` (each single-path refusal) is ConflictedPathRefused naming the first such file in order;
/// - anything else is ConflictedPathRefused naming no path.
/// ConflictedPathRefused keeps the refusal's message as its reason.
pub(crate) fn tree_refusal(error: GitError, files: &[PathBuf]) -> JoinRunError {
    if error.code.is_some() || !error.args.is_empty() {
        return JoinRunError::Git(error);
    }
    let reason = error.stderr;
    if let Some(path) = files.iter().find(|file| reason == format!("not a file entry: {file:?} is a folder")) {
        return JoinRunError::ConflictedPathIsFolder { path: path.clone() };
    }
    let in_overlap = |file: &PathBuf| {
        files.iter().any(|other| {
            reason.ends_with(&format!(": {file:?} and {other:?}"))
                || reason.ends_with(&format!(": {other:?} and {file:?}"))
        })
    };
    let path = files
        .iter()
        .find(|file| in_overlap(file))
        .or_else(|| files.iter().find(|file| reason.ends_with(&format!(": {file:?}"))))
        .cloned();
    JoinRunError::ConflictedPathRefused { path, reason }
}
