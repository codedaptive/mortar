//! Writes a folder's record and its partners into its git dir, whole or not at all.
//! No unit test: it writes and renames real files; encode_record_and_partners_tests covers the text.
//! tests/folders/partner_base.rs covers the file holding the encoded record and one newline, with partners and
//! without, and a rewrite replacing it. The integration group is to cover: no temp file is left beside it.

use std::path::Path;

use super::FolderError;
use crate::record::{encode_record_and_partners, record_path, FolderRecord, RequestPartner};

/// Writes encode_record_and_partners(`record`, `partners`) and "\n" to `mortar-folder.json.tmp` beside
/// record_path(`git_dir`), then renames it over the record, so a reader finds the old record or the new one, never
/// a torn one. With no partners the file is encode_folder_record's line, byte for byte.
pub(crate) fn write_folder_record(
    git_dir: &Path,
    record: &FolderRecord,
    partners: &[RequestPartner],
) -> Result<(), FolderError> {
    let path = record_path(git_dir);
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, format!("{}\n", encode_record_and_partners(record, partners)))
        .map_err(|source| FolderError::Io { path: temp.clone(), source })?;
    std::fs::rename(&temp, &path).map_err(|source| FolderError::Io { path, source })
}
