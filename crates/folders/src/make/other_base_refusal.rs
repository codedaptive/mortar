//! Does a repeat of make_request_folder_on ask for another base or other partners than the folder records?

use std::path::Path;

use super::FolderError;
use crate::record::{FolderRecord, RecordedBase, RequestPartner};

/// The refusal for a repeat that wants `wanted` where the folder at `path` records `existing` with
/// `existing_partners`; the caller has found the two records the same folder (repeat_decision). Another base is
/// BranchExists naming `branch` (a short name): the folder's branch was cut at the recorded base, so it is not
/// made at the wanted one. The same base with other partners (another tip, order or count) is
/// DifferentFolder naming `path` and `existing`. None when base and partners both match.
pub(crate) fn other_base_refusal(
    wanted: &RecordedBase,
    existing: &FolderRecord,
    existing_partners: &[RequestPartner],
    branch: &str,
    path: &Path,
) -> Option<FolderError> {
    if existing.base != wanted.base {
        return Some(FolderError::BranchExists { branch: branch.to_string() });
    }
    if existing_partners != wanted.partners.as_slice() {
        return Some(FolderError::DifferentFolder {
            path: path.to_path_buf(),
            existing: existing.clone(),
        });
    }
    None
}
