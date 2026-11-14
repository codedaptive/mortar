//! read_tree_entry: read the answer of `ls-tree -z --full-tree <tree> -- <path>`.

use crate::read_tree_entry_record::read_tree_entry_record;
use crate::{EntryKind, Oid, TreeEntryError};
use std::path::Path;

/// Empty output is a path the tree lacks (None). Otherwise read_tree_entry_record checks the output is the one
/// record for `path` with a valid id, refused as Git when it is not; those checks come before the type. Then a `blob`
/// (a file or a symlink) is its id; any other type is refused as NotABlob carrying the path and the kind (`commit`
/// Commit, `tree` Tree, else Other).
pub(crate) fn read_tree_entry(stdout: &[u8], path: &Path) -> Result<Option<Oid>, TreeEntryError> {
    let Some((kind, oid)) = read_tree_entry_record(stdout, path).map_err(TreeEntryError::Git)? else {
        return Ok(None);
    };
    let kind = match kind {
        b"blob" => return Ok(Some(oid)),
        b"commit" => EntryKind::Commit,
        b"tree" => EntryKind::Tree,
        other => EntryKind::Other(String::from_utf8_lossy(other).into_owned()),
    };
    Err(TreeEntryError::NotABlob { path: path.to_path_buf(), kind })
}
