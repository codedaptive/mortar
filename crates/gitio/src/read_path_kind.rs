//! read_path_kind: the kind of entry one ls-tree record names.

use crate::read_tree_entry_record::read_tree_entry_record;
use crate::{GitError, PathKind};
use std::path::Path;

/// The answer of `ls-tree -z --full-tree <tree> -- <path>` read as a kind: None for empty output (the tree lacks the
/// path), File for type `blob`, Folder for `tree`, Gitlink for `commit`. Any other type is a GitError with no args and
/// no code naming the path and the type. Refusals of the record itself (not one record, bad fields, another path, a
/// bad id) come from read_tree_entry_record.
pub(crate) fn read_path_kind(stdout: &[u8], path: &Path) -> Result<Option<PathKind>, GitError> {
    let Some((kind, _)) = read_tree_entry_record(stdout, path)? else {
        return Ok(None);
    };
    match kind {
        b"blob" => Ok(Some(PathKind::File)),
        b"tree" => Ok(Some(PathKind::Folder)),
        b"commit" => Ok(Some(PathKind::Gitlink)),
        _ => Err(GitError::refusal(
            Vec::new(),
            format!("not a blob, tree or commit: {path:?} is a {:?}", String::from_utf8_lossy(kind)),
        )),
    }
}
